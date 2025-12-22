// rax25kb - Cross-Connect Configuration Module
// Part 1: Configuration structures for multiple serial ports and cross-connects

use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq)]
enum FlowControl {
    None,
    Software,
    Hardware,
    DtrDsr,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StopBits {
    One,
    Two,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Parity {
    None,
    Odd,
    Even,
}

#[derive(Debug, Clone, PartialEq)]
enum CrossConnectEndpoint {
    TcpSocket {
        address: String,
        port: u16,
    },
    SerialPort {
        port_id: String,
        kiss_port: u8,  // KISS TNC port number (0-15)
    },
}

#[derive(Debug, Clone)]
struct SerialPortConfig {
    id: String,
    device: String,
    baud_rate: u32,
    flow_control: FlowControl,
    stop_bits: StopBits,
    parity: Parity,
    extended_kiss: bool,  // true for XKISS, false for standard KISS
}

#[derive(Debug, Clone)]
struct CrossConnect {
    id: String,
    endpoint_a: CrossConnectEndpoint,
    endpoint_b: CrossConnectEndpoint,
    phil_flag: bool,
    dump_frames: bool,
    parse_kiss: bool,
    dump_ax25: bool,
    raw_copy: bool,
}

#[derive(Debug, Clone)]
struct Config {
    serial_ports: HashMap<String, SerialPortConfig>,
    cross_connects: Vec<CrossConnect>,
    log_level: u8,
    logfile: Option<String>,
    pidfile: Option<String>,
    log_to_console: bool,
    quiet_startup: bool,
    pcap_file: Option<String>,
}

impl Config {
    fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file '{}': {}", path, e))?;
        
        let mut config_map = HashMap::new();
        
        // Parse config file into key-value map
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let mut value = value.trim();
                if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                    value = &value[1..value.len()-1];
                }
                config_map.insert(key.to_string(), value.to_string());
            }
        }
        
        // Parse serial port configurations
        let mut serial_ports = HashMap::new();
        let mut serial_port_ids = Vec::new();
        
        // Find all serial_portXXXX entries
        for key in config_map.keys() {
            if key.starts_with("serial_port") && key.len() > 11 {
                // Extract ID (e.g., "serial_port0000" -> "0000")
                let id = &key[11..];
                if !serial_port_ids.contains(&id.to_string()) {
                    serial_port_ids.push(id.to_string());
                }
            }
        }
        
        // Parse each serial port configuration
        for id in serial_port_ids {
            let device_key = format!("serial_port{}", id);
            let device = config_map.get(&device_key)
                .ok_or(format!("Missing device for serial port {}", id))?
                .clone();
            
            let baud_key = format!("serial_port{}_baud", id);
            let baud_rate = config_map.get(&baud_key)
                .and_then(|v| v.parse().ok())
                .unwrap_or(9600);
            
            let flow_key = format!("serial_port{}_flow_control", id);
            let flow_control = config_map.get(&flow_key)
                .and_then(|v| Self::parse_flow_control(v))
                .unwrap_or(FlowControl::None);
            
            let stop_key = format!("serial_port{}_stop_bits", id);
            let stop_bits = config_map.get(&stop_key)
                .and_then(|v| Self::parse_stop_bits(v))
                .unwrap_or(StopBits::One);
            
            let parity_key = format!("serial_port{}_parity", id);
            let parity = config_map.get(&parity_key)
                .and_then(|v| Self::parse_parity(v))
                .unwrap_or(Parity::None);
            
            let xkiss_key = format!("serial_port{}_extended_kiss", id);
            let extended_kiss = config_map.get(&xkiss_key)
                .and_then(|v| Self::parse_bool(v))
                .unwrap_or(false);
            
            let port_config = SerialPortConfig {
                id: id.clone(),
                device,
                baud_rate,
                flow_control,
                stop_bits,
                parity,
                extended_kiss,
            };
            
            serial_ports.insert(id, port_config);
        }
        
        // Parse cross-connect configurations
        let mut cross_connects = Vec::new();
        let mut cross_connect_ids = Vec::new();
        
        // Find all cross_connectXXXX entries
        for key in config_map.keys() {
            if key.starts_with("cross_connect") && key.len() == 17 {
                // Extract ID (e.g., "cross_connect0000" -> "0000")
                let id = &key[13..17];
                if !cross_connect_ids.contains(&id.to_string()) {
                    cross_connect_ids.push(id.to_string());
                }
            }
        }
        
        // Sort IDs to process in order
        cross_connect_ids.sort();
        
        // Parse each cross-connect configuration
        for id in cross_connect_ids {
            let cc_key = format!("cross_connect{}", id);
            let cc_value = config_map.get(&cc_key)
                .ok_or(format!("Missing cross_connect{}", id))?;
            
            // Parse cross-connect value: "endpoint_a <-> endpoint_b"
            let parts: Vec<&str> = cc_value.split("<->").collect();
            if parts.len() != 2 {
                return Err(format!("Invalid cross_connect{} format: {}", id, cc_value).into());
            }
            
            let endpoint_a = Self::parse_endpoint(parts[0].trim(), &serial_ports)?;
            let endpoint_b = Self::parse_endpoint(parts[1].trim(), &serial_ports)?;
            
            // Parse optional flags for this cross-connect
            let phil_key = format!("cross_connect{}_phil_flag", id);
            let phil_flag = config_map.get(&phil_key)
                .and_then(|v| Self::parse_bool(v))
                .unwrap_or(false);
            
            let dump_key = format!("cross_connect{}_dump", id);
            let dump_frames = config_map.get(&dump_key)
                .and_then(|v| Self::parse_bool(v))
                .unwrap_or(false);
            
            let parse_key = format!("cross_connect{}_parse_kiss", id);
            let parse_kiss = config_map.get(&parse_key)
                .and_then(|v| Self::parse_bool(v))
                .unwrap_or(false);
            
            let ax25_key = format!("cross_connect{}_dump_ax25", id);
            let dump_ax25 = config_map.get(&ax25_key)
                .and_then(|v| Self::parse_bool(v))
                .unwrap_or(false);
            
            let raw_key = format!("cross_connect{}_raw_copy", id);
            let raw_copy = config_map.get(&raw_key)
                .and_then(|v| Self::parse_bool(v))
                .unwrap_or(false);
            
            let cross_connect = CrossConnect {
                id: id.clone(),
                endpoint_a,
                endpoint_b,
                phil_flag,
                dump_frames,
                parse_kiss,
                dump_ax25,
                raw_copy,
            };
            
            cross_connects.push(cross_connect);
        }
        
        // If no cross-connects defined, create default
        if cross_connects.is_empty() && !serial_ports.is_empty() {
            // Create default cross_connect0000
            let first_port_id = serial_ports.keys().next().unwrap().clone();
            let default_cc = CrossConnect {
                id: "0000".to_string(),
                endpoint_a: CrossConnectEndpoint::SerialPort {
                    port_id: first_port_id,
                    kiss_port: 0,
                },
                endpoint_b: CrossConnectEndpoint::TcpSocket {
                    address: "0.0.0.0".to_string(),
                    port: 8001,
                },
                phil_flag: false,
                dump_frames: false,
                parse_kiss: false,
                dump_ax25: false,
                raw_copy: false,
            };
            cross_connects.push(default_cc);
        }
        
        // Parse global settings
        let log_level = config_map.get("log_level")
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        
        let logfile = config_map.get("logfile").cloned();
        let pidfile = config_map.get("pidfile").cloned();
        let pcap_file = config_map.get("pcap_file").cloned();
        
        let log_to_console = config_map.get("log_to_console")
            .and_then(|v| Self::parse_bool(v))
            .unwrap_or(true);
        
        let quiet_startup = config_map.get("quiet_startup")
            .and_then(|v| Self::parse_bool(v))
            .unwrap_or(false);
        
        Ok(Config {
            serial_ports,
            cross_connects,
            log_level,
            logfile,
            pidfile,
            log_to_console,
            quiet_startup,
            pcap_file,
        })
    }
    
    fn parse_endpoint(s: &str, serial_ports: &HashMap<String, SerialPortConfig>) 
        -> Result<CrossConnectEndpoint, Box<dyn std::error::Error>> {
        
        // Format: "tcp:address:port" or "serial:port_id:kiss_port"
        let parts: Vec<&str> = s.split(':').collect();
        
        match parts[0] {
            "tcp" => {
                if parts.len() != 3 {
                    return Err(format!("Invalid TCP endpoint format: {}", s).into());
                }
                let address = parts[1].to_string();
                let port = parts[2].parse::<u16>()
                    .map_err(|_| format!("Invalid TCP port: {}", parts[2]))?;
                
                Ok(CrossConnectEndpoint::TcpSocket { address, port })
            }
            "serial" => {
                if parts.len() != 3 {
                    return Err(format!("Invalid serial endpoint format: {}", s).into());
                }
                let port_id = parts[1].to_string();
                let kiss_port = parts[2].parse::<u8>()
                    .map_err(|_| format!("Invalid KISS port: {}", parts[2]))?;
                
                if kiss_port > 15 {
                    return Err(format!("KISS port must be 0-15, got: {}", kiss_port).into());
                }
                
                if !serial_ports.contains_key(&port_id) {
                    return Err(format!("Unknown serial port ID: {}", port_id).into());
                }
                
                Ok(CrossConnectEndpoint::SerialPort { port_id, kiss_port })
            }
            _ => Err(format!("Invalid endpoint type: {}", parts[0]).into()),
        }
    }
    
    fn parse_flow_control(s: &str) -> Option<FlowControl> {
        match s.to_lowercase().as_str() {
            "software" | "xon" | "xonxoff" | "xon-xoff" => Some(FlowControl::Software),
            "hardware" | "rtscts" | "rts-cts" | "rts/cts" => Some(FlowControl::Hardware),
            "dtrdsr" | "dtr-dsr" | "dtr/dsr" => Some(FlowControl::DtrDsr),
            "none" | "off" | "no" => Some(FlowControl::None),
            _ => None,
        }
    }
    
    fn parse_stop_bits(s: &str) -> Option<StopBits> {
        match s.as_str() {
            "1" | "one" => Some(StopBits::One),
            "2" | "two" => Some(StopBits::Two),
            _ => None,
        }
    }
    
    fn parse_parity(s: &str) -> Option<Parity> {
        match s.to_lowercase().as_str() {
            "none" | "n" | "no" => Some(Parity::None),
            "odd" | "o" => Some(Parity::Odd),
            "even" | "e" => Some(Parity::Even),
            _ => None,
        }
    }
    
    fn parse_bool(s: &str) -> Option<bool> {
        match s.to_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        }
    }
}

// Example configuration file format:
//
// # Serial port definitions
// serial_port0000=/dev/ttyUSB0
// serial_port0000_baud=9600
// serial_port0000_flow_control=none
// serial_port0000_stop_bits=1
// serial_port0000_parity=none
// serial_port0000_extended_kiss=false
//
// serial_port0001=/dev/ttyUSB1
// serial_port0001_baud=9600
// serial_port0001_extended_kiss=true
//
// # Cross-connect definitions
// # Format: endpoint_a <-> endpoint_b
// # TCP endpoint: tcp:address:port
// # Serial endpoint: serial:port_id:kiss_port_number
//
// # Serial port 0000, KISS port 0 <-> TCP socket
// cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
// cross_connect0000_parse_kiss=true
//
// # Serial port 0000, KISS port 1 <-> Serial port 0001, KISS port 0
// # (translates between standard KISS port 1 and XKISS port 0)
// cross_connect0001=serial:0000:1 <-> serial:0001:0
// cross_connect0001_parse_kiss=true
//
// # Global settings
// log_level=5
// logfile=/var/log/rax25kb.log
// log_to_console=true