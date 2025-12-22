// rax25kb - Cross-Connect Manager Module
// Part 3: Managing multiple cross-connects with serial and TCP endpoints

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serialport::SerialPort;

// Import from previous modules
use crate::{
    Config, CrossConnect, CrossConnectEndpoint, SerialPortConfig,
    KissPortTranslator, KissFrameBuffer, FrameRouter,
    Logger, PcapWriter,
};

const KISS_FEND: u8 = 0xC0;

/// Manages all serial ports
pub struct SerialPortManager {
    ports: HashMap<String, Arc<Mutex<Box<dyn SerialPort>>>>,
}

impl SerialPortManager {
    pub fn new() -> Self {
        SerialPortManager {
            ports: HashMap::new(),
        }
    }
    
    /// Open a serial port with the given configuration
    pub fn open_port(&mut self, config: &SerialPortConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut port_builder = serialport::new(&config.device, config.baud_rate)
            .timeout(Duration::from_millis(100));
        
        port_builder = match config.flow_control {
            FlowControl::None => port_builder.flow_control(serialport::FlowControl::None),
            FlowControl::Software => port_builder.flow_control(serialport::FlowControl::Software),
            FlowControl::Hardware => port_builder.flow_control(serialport::FlowControl::Hardware),
            FlowControl::DtrDsr => {
                #[cfg(target_os = "windows")]
                { port_builder.flow_control(serialport::FlowControl::Hardware) }
                #[cfg(not(target_os = "windows"))]
                { port_builder.flow_control(serialport::FlowControl::None) }
            }
        };
        
        port_builder = match config.stop_bits {
            StopBits::One => port_builder.stop_bits(serialport::StopBits::One),
            StopBits::Two => port_builder.stop_bits(serialport::StopBits::Two),
        };
        
        port_builder = match config.parity {
            Parity::None => port_builder.parity(serialport::Parity::None),
            Parity::Odd => port_builder.parity(serialport::Parity::Odd),
            Parity::Even => port_builder.parity(serialport::Parity::Even),
        };
        
        let port = port_builder.open()?;
        self.ports.insert(config.id.clone(), Arc::new(Mutex::new(port)));
        
        Ok(())
    }
    
    /// Get a cloned reference to a serial port
    pub fn get_port(&self, id: &str) -> Option<Arc<Mutex<Box<dyn SerialPort>>>> {
        self.ports.get(id).map(|p| Arc::clone(p))
    }
}

/// Represents an active cross-connect connection
pub struct ActiveConnection {
    id: String,
    _handle: thread::JoinHandle<()>,
}

/// Manages all cross-connects
pub struct CrossConnectManager {
    config: Arc<Config>,
    serial_manager: Arc<Mutex<SerialPortManager>>,
    logger: Arc<Logger>,
    pcap_writer: Option<Arc<PcapWriter>>,
    connections: Vec<ActiveConnection>,
}

impl CrossConnectManager {
    pub fn new(
        config: Config,
        logger: Arc<Logger>,
        pcap_writer: Option<Arc<PcapWriter>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut serial_manager = SerialPortManager::new();
        
        // Open all serial ports
        for (id, port_config) in &config.serial_ports {
            logger.log(&format!("Opening serial port {}: {}", id, port_config.device), 5);
            serial_manager.open_port(port_config)?;
        }
        
        Ok(CrossConnectManager {
            config: Arc::new(config),
            serial_manager: Arc::new(Mutex::new(serial_manager)),
            logger,
            pcap_writer,
            connections: Vec::new(),
        })
    }
    
    /// Start all cross-connects
    pub fn start_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for cross_connect in &self.config.cross_connects {
            self.logger.log(
                &format!("Starting cross-connect {}: {:?} <-> {:?}", 
                    cross_connect.id, cross_connect.endpoint_a, cross_connect.endpoint_b),
                5
            );
            
            self.start_cross_connect(cross_connect)?;
        }
        
        Ok(())
    }
    
    /// Start a specific cross-connect
    fn start_cross_connect(&mut self, cc: &CrossConnect) -> Result<(), Box<dyn std::error::Error>> {
        match (&cc.endpoint_a, &cc.endpoint_b) {
            // Serial to TCP
            (CrossConnectEndpoint::SerialPort { port_id, kiss_port }, 
             CrossConnectEndpoint::TcpSocket { address, port }) |
            (CrossConnectEndpoint::TcpSocket { address, port },
             CrossConnectEndpoint::SerialPort { port_id, kiss_port }) => {
                self.start_serial_to_tcp(cc, port_id, *kiss_port, address, *port)?;
            }
            
            // Serial to Serial
            (CrossConnectEndpoint::SerialPort { port_id: port_a, kiss_port: kiss_a },
             CrossConnectEndpoint::SerialPort { port_id: port_b, kiss_port: kiss_b }) => {
                self.start_serial_to_serial(cc, port_a, *kiss_a, port_b, *kiss_b)?;
            }
            
            // TCP to TCP (not implemented)
            (CrossConnectEndpoint::TcpSocket { .. }, CrossConnectEndpoint::TcpSocket { .. }) => {
                return Err("TCP to TCP cross-connects not supported".into());
            }
        }
        
        Ok(())
    }
    
    /// Start a serial-to-TCP cross-connect
    fn start_serial_to_tcp(
        &mut self,
        cc: &CrossConnect,
        serial_id: &str,
        kiss_port: u8,
        tcp_address: &str,
        tcp_port: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bind_address = format!("{}:{}", tcp_address, tcp_port);
        let listener = TcpListener::bind(&bind_address)?;
        
        self.logger.log(&format!("Cross-connect {} listening on {}", cc.id, bind_address), 5);
        
        let serial_manager = Arc::clone(&self.serial_manager);
        let serial_id = serial_id.to_string();
        let cc_config = cc.clone();
        let logger = Arc::clone(&self.logger);
        let pcap_writer = self.pcap_writer.clone();
        let config = Arc::clone(&self.config);
        
        let handle = thread::spawn(move || {
            loop {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        logger.log(&format!("Cross-connect {}: Client connected from {}", cc_config.id, addr), 5);
                        
                        let serial_port = {
                            let mgr = serial_manager.lock().unwrap();
                            mgr.get_port(&serial_id)
                        };
                        
                        if let Some(port) = serial_port {
                            Self::handle_serial_tcp_connection(
                                stream,
                                port,
                                kiss_port,
                                &cc_config,
                                &logger,
                                &pcap_writer,
                                &config,
                            );
                        } else {
                            logger.log(&format!("Serial port {} not found", serial_id), 3);
                        }
                    }
                    Err(e) => {
                        logger.log(&format!("Accept error on cross-connect {}: {}", cc_config.id, e), 3);
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        });
        
        self.connections.push(ActiveConnection {
            id: cc.id.clone(),
            _handle: handle,
        });
        
        Ok(())
    }
    
    /// Handle a serial-to-TCP connection
    fn handle_serial_tcp_connection(
        mut stream: TcpStream,
        serial_port: Arc<Mutex<Box<dyn SerialPort>>>,
        kiss_port: u8,
        cc_config: &CrossConnect,
        logger: &Arc<Logger>,
        pcap_writer: &Option<Arc<PcapWriter>>,
        config: &Arc<Config>,
    ) {
        if cc_config.raw_copy {
            Self::handle_raw_copy(stream, serial_port, logger);
            return;
        }
        
        let serial_clone = Arc::clone(&serial_port);
        let mut read_stream = stream.try_clone().expect("Failed to clone stream");
        let logger_clone = Arc::clone(logger);
        let cc_config_clone = cc_config.clone();
        let pcap_clone = pcap_writer.clone();
        
        // Serial to TCP thread
        let serial_to_tcp = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            let mut frame_buffer = KissFrameBuffer::new();
            
            loop {
                let mut port = serial_clone.lock().unwrap();
                match port.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        let frames = frame_buffer.add_bytes(&buffer[..n]);
                        
                        for frame in frames {
                            // Check if frame is for our KISS port
                            if let Some((port_num, _cmd, _idx)) = extract_kiss_info(&frame) {
                                if port_num == kiss_port {
                                    // Process and forward frame
                                    let processed = if cc_config_clone.phil_flag {
                                        process_frame_with_phil_flag(&frame)
                                    } else {
                                        frame
                                    };
                                    
                                    if cc_config_clone.parse_kiss {
                                        parse_and_log_kiss(&processed, "Serial->TCP", &pcap_clone);
                                    }
                                    
                                    if let Err(e) = read_stream.write_all(&processed) {
                                        logger_clone.log(&format!("Error writing to TCP: {}", e), 3);
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Ok(_) => thread::sleep(Duration::from_millis(10)),
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        logger_clone.log(&format!("Error reading from serial: {}", e), 3);
                        break;
                    }
                }
            }
        });
        
        // TCP to Serial loop
        let mut buffer = [0u8; 1024];
        let mut frame_buffer = KissFrameBuffer::new();
        
        loop {
            match stream.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let frames = frame_buffer.add_bytes(&buffer[..n]);
                    
                    for frame in frames {
                        // Modify frame to target the correct KISS port
                        let modified = modify_kiss_port(&frame, kiss_port);
                        
                        let processed = if cc_config.phil_flag {
                            process_phil_flag_tcp_to_serial(&modified)
                        } else {
                            modified
                        };
                        
                        if cc_config.parse_kiss {
                            parse_and_log_kiss(&processed, "TCP->Serial", pcap_writer);
                        }
                        
                        let mut port = serial_port.lock().unwrap();
                        if let Err(e) = port.write_all(&processed) {
                            logger.log(&format!("Error writing to serial: {}", e), 3);
                            break;
                        }
                    }
                }
                Ok(_) => {
                    logger.log("Client disconnected", 5);
                    break;
                }
                Err(e) => {
                    logger.log(&format!("Error reading from TCP: {}", e), 3);
                    break;
                }
            }
        }
        
        drop(serial_to_tcp);
        logger.log("Connection closed", 5);
    }
    
    /// Start a serial-to-serial cross-connect
    fn start_serial_to_serial(
        &mut self,
        cc: &CrossConnect,
        serial_a_id: &str,
        kiss_port_a: u8,
        serial_b_id: &str,
        kiss_port_b: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let serial_manager = Arc::clone(&self.serial_manager);
        
        let port_a = {
            let mgr = serial_manager.lock().unwrap();
            mgr.get_port(serial_a_id)
        }.ok_or(format!("Serial port {} not found", serial_a_id))?;
        
        let port_b = {
            let mgr = serial_manager.lock().unwrap();
            mgr.get_port(serial_b_id)
        }.ok_or(format!("Serial port {} not found", serial_b_id))?;
        
        // Get extended KISS settings
        let config = Arc::clone(&self.config);
        let extended_a = config.serial_ports.get(serial_a_id)
            .map(|p| p.extended_kiss)
            .unwrap_or(false);
        let extended_b = config.serial_ports.get(serial_b_id)
            .map(|p| p.extended_kiss)
            .unwrap_or(false);
        
        let translator_a_to_b = KissPortTranslator::new(kiss_port_a, kiss_port_b, extended_a, extended_b);
        let translator_b_to_a = KissPortTranslator::new(kiss_port_b, kiss_port_a, extended_b, extended_a);
        
        let cc_config = cc.clone();
        let logger = Arc::clone(&self.logger);
        let pcap_writer = self.pcap_writer.clone();
        
        let port_a_clone = Arc::clone(&port_a);
        let port_b_clone = Arc::clone(&port_b);
        let logger_a = Arc::clone(&logger);
        let cc_a = cc_config.clone();
        let pcap_a = pcap_writer.clone();
        
        // Thread A to B
        let a_to_b = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            let mut frame_buffer = KissFrameBuffer::new();
            
            loop {
                let mut port = port_a_clone.lock().unwrap();
                match port.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        drop(port);
                        let frames = frame_buffer.add_bytes(&buffer[..n]);
                        
                        for frame in frames {
                            if let Some(translated) = translator_a_to_b.translate(&frame) {
                                if cc_a.parse_kiss {
                                    parse_and_log_kiss(&translated, 
                                        &format!("Serial {}:{} -> Serial {}:{}", 
                                            serial_a_id, kiss_port_a, serial_b_id, kiss_port_b),
                                        &pcap_a);
                                }
                                
                                let mut port_b = port_b_clone.lock().unwrap();
                                if let Err(e) = port_b.write_all(&translated) {
                                    logger_a.log(&format!("Error writing to serial B: {}", e), 3);
                                    break;
                                }
                            }
                        }
                    }
                    Ok(_) => {
                        drop(port);
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        drop(port);
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        logger_a.log(&format!("Error reading from serial A: {}", e), 3);
                        break;
                    }
                }
            }
        });
        
        // Thread B to A
        let b_to_a = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            let mut frame_buffer = KissFrameBuffer::new();
            
            loop {
                let mut port = port_b.lock().unwrap();
                match port.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        drop(port);
                        let frames = frame_buffer.add_bytes(&buffer[..n]);
                        
                        for frame in frames {
                            if let Some(translated) = translator_b_to_a.translate(&frame) {
                                if cc_config.parse_kiss {
                                    parse_and_log_kiss(&translated,
                                        &format!("Serial {}:{} -> Serial {}:{}", 
                                            serial_b_id, kiss_port_b, serial_a_id, kiss_port_a),
                                        &pcap_writer);
                                }
                                
                                let mut port_a = port_a.lock().unwrap();
                                if let Err(e) = port_a.write_all(&translated) {
                                    logger.log(&format!("Error writing to serial A: {}", e), 3);
                                    break;
                                }
                            }
                        }
                    }
                    Ok(_) => {
                        drop(port);
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        drop(port);
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        logger.log(&format!("Error reading from serial B: {}", e), 3);
                        break;
                    }
                }
            }
        });
        
        // Store handles (in production, you'd want to properly manage these)
        let handle = thread::spawn(move || {
            let _ = a_to_b.join();
            let _ = b_to_a.join();
        });
        
        self.connections.push(ActiveConnection {
            id: cc.id.clone(),
            _handle: handle,
        });
        
        Ok(())
    }
    
    /// Handle raw copy mode (no KISS processing)
    fn handle_raw_copy(
        mut stream: TcpStream,
        serial_port: Arc<Mutex<Box<dyn SerialPort>>>,
        logger: &Arc<Logger>,
    ) {
        let serial_clone = Arc::clone(&serial_port);
        let mut read_stream = stream.try_clone().expect("Failed to clone stream");
        let logger_clone = Arc::clone(logger);
        
        let serial_to_tcp = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                let mut port = serial_clone.lock().unwrap();
                match port.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        if let Err(e) = read_stream.write_all(&buffer[..n]) {
                            logger_clone.log(&format!("Raw copy: Error writing to TCP: {}", e), 3);
                            break;
                        }
                    }
                    Ok(_) => thread::sleep(Duration::from_millis(10)),
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        logger_clone.log(&format!("Raw copy: Error reading from serial: {}", e), 3);
                        break;
                    }
                }
            }
        });

        let mut buffer = [0u8; 1024];
        loop {
            match stream.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let mut port = serial_port.lock().unwrap();
                    if let Err(e) = port.write_all(&buffer[..n]) {
                        logger.log(&format!("Raw copy: Error writing to serial: {}", e), 3);
                        break;
                    }
                }
                Ok(_) => {
                    logger.log("Raw copy: Client disconnected", 5);
                    break;
                }
                Err(e) => {
                    logger.log(&format!("Raw copy: Error reading from TCP: {}", e), 3);
                    break;
                }
            }
        }
        
        drop(serial_to_tcp);
    }
    
    /// Keep the manager running
    pub fn run_forever(&self) {
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    }
}

// Helper functions (would be imported from other modules in practice)
fn extract_kiss_info(frame: &[u8]) -> Option<(u8, u8, usize)> {
    if frame.len() < 2 || frame[0] != KISS_FEND {
        return None;
    }
    let cmd_byte = frame[1];
    let port = (cmd_byte >> 4) & 0x0F;
    let command = cmd_byte & 0x0F;
    Some((port, command, 2))
}

fn modify_kiss_port(frame: &[u8], new_port: u8) -> Vec<u8> {
    if frame.len() < 2 || frame[0] != KISS_FEND {
        return frame.to_vec();
    }
    let mut result = frame.to_vec();
    let cmd_byte = frame[1];
    let command = cmd_byte & 0x0F;
    result[1] = ((new_port & 0x0F) << 4) | command;
    result
}

fn process_frame_with_phil_flag(frame: &[u8]) -> Vec<u8> {
    // Placeholder - would call actual implementation
    frame.to_vec()
}

fn process_phil_flag_tcp_to_serial(data: &[u8]) -> Vec<u8> {
    // Placeholder - would call actual implementation
    data.to_vec()
}

fn parse_and_log_kiss(frame: &[u8], direction: &str, pcap: &Option<Arc<PcapWriter>>) {
    // Placeholder - would call actual implementation
    println!("KISS frame: {} bytes, direction: {}", frame.len(), direction);
}