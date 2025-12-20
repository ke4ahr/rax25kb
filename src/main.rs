// rax25kb - AX.25 KISS Bridge
//
// Copyright (C) 2025 Kris Kirby
//
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rax25kb main source
//
// This file is part of rax25kb.
//
// rax25kb is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rax25kb is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rax25kb.  If not, see <https://www.gnu.org/licenses/>.
//
// This program exists to correct packets which have been improperly passed by a TNC from 
// raw AX.25 to KISS framing without proper byte swaps as required by the KISS standard 
// where the character 0xC0h is involved. In KISS encapsulation, the packet is wrapped in 
// SLIP-like headers and 0xC0h is converted to 0xDBh which is normally changed to the 
// FESC TFEND (0xDBh 0xDCh) sequence. This bug is present in some radios using the TASCO 
// modem chipset. This code also implements a sequence to prevent the radio from parsing 
// TC0\n by converting the 'C' (0x43h or 67) character to a FESC (0xDBh), 0x43h sequence. 
// See the included documentation for more information.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serialport::{SerialPort, SerialPortBuilder};

const KISS_FEND: u8 = 0xC0;
const KISS_FESC: u8 = 0xDB;
const KISS_TFEND: u8 = 0xDC;
const KISS_TFESC: u8 = 0xDD;

#[derive(Debug, Clone)]
struct Config {
    serial_port: String,
    baud_rate: u32,
    flow_control: FlowControl,
    stop_bits: StopBits,
    parity: Parity,
    tcp_addresses: Vec<String>,
    tcp_port: u16,
    phil_flag: bool,
    dump_frames: bool,
    parse_kiss: bool,
    dump_ax25: bool,
    log_level: u8,
    logfile: Option<String>,
    pidfile: Option<String>,
    log_to_console: bool,
    log_to_file_only: bool,
    ipv4_only: bool,
    ipv6_only: bool,
    quiet_startup: bool,
    pcap_file: Option<String>,
    raw_copy: bool,
}

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

impl Config {
    fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file '{}': {}", path, e))?;
        
        let mut config_map = HashMap::new();
        
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
        
        let serial_port = config_map.get("serial_port")
            .ok_or("Missing required config: serial_port")?
            .clone();
        
        let baud_rate = config_map.get("baud_rate")
            .and_then(|v| v.parse().ok())
            .unwrap_or(9600);
        
        let flow_control = config_map.get("flow_control")
            .and_then(|v| match v.to_lowercase().as_str() {
                "software" | "xon" | "xonxoff" | "xon-xoff" => Some(FlowControl::Software),
                "hardware" | "rtscts" | "rts-cts" | "rts/cts" => Some(FlowControl::Hardware),
                "dtrdsr" | "dtr-dsr" | "dtr/dsr" => Some(FlowControl::DtrDsr),
                "none" | "off" | "no" => Some(FlowControl::None),
                _ => None
            })
            .unwrap_or(FlowControl::None);
        
        let stop_bits = config_map.get("stop_bits")
            .and_then(|v| match v.as_str() {
                "1" | "one" => Some(StopBits::One),
                "2" | "two" => Some(StopBits::Two),
                _ => None
            })
            .unwrap_or(StopBits::One);
        
        let parity = config_map.get("parity")
            .and_then(|v| match v.to_lowercase().as_str() {
                "none" | "n" | "no" => Some(Parity::None),
                "odd" | "o" => Some(Parity::Odd),
                "even" | "e" => Some(Parity::Even),
                _ => None
            })
            .unwrap_or(Parity::None);
        
        let tcp_addresses = config_map.get("tcp_address")
            .map(|addr_str| {
                addr_str
                    .split(|c| c == ',' || c == ' ')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            })
            .filter(|v: &Vec<String>| !v.is_empty())
            .unwrap_or_else(|| vec!["0.0.0.0".to_string()]);
        
        let tcp_port = config_map.get("tcp_port")
            .and_then(|v| v.parse().ok())
            .unwrap_or(8001);
        
        let phil_flag = config_map.get("phil_flag")
            .and_then(|v| match v.to_lowercase().as_str() {
                "1" | "true" | "yes" => Some(true),
                _ => Some(false)
            })
            .unwrap_or(false);
        
        let dump_frames = config_map.get("dump")
            .and_then(|v| match v.to_lowercase().as_str() {
                "1" | "true" | "yes" => Some(true),
                _ => Some(false)
            })
            .unwrap_or(false);
        
        let parse_kiss = config_map.get("parse_kiss")
            .and_then(|v| match v.to_lowercase().as_str() {
                "1" | "true" | "yes" => Some(true),
                _ => Some(false)
            })
            .unwrap_or(false);
        
        let dump_ax25 = config_map.get("dump_ax25")
            .and_then(|v| match v.to_lowercase().as_str() {
                "1" | "true" | "yes" => Some(true),
                _ => Some(false)
            })
            .unwrap_or(false);
        
        let log_level = config_map.get("log_level")
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        
        let raw_copy = config_map.get("raw_copy")
            .and_then(|v| match v.to_lowercase().as_str() {
                "1" | "true" | "yes" => Some(true),
                _ => Some(false)
            })
            .unwrap_or(false);
        
        let logfile = config_map.get("logfile").cloned();
        let pidfile = config_map.get("pidfile").cloned();
        let pcap_file = config_map.get("pcap_file").cloned();
        
        Ok(Config {
            serial_port,
            baud_rate,
            flow_control,
            stop_bits,
            parity,
            tcp_addresses,
            tcp_port,
            phil_flag,
            dump_frames,
            parse_kiss,
            dump_ax25,
            log_level,
            logfile,
            pidfile,
            log_to_console: true,
            log_to_file_only: false,
            ipv4_only: false,
            ipv6_only: false,
            quiet_startup: false,
            pcap_file,
            raw_copy,
        })
    }
    
    fn apply_cli_overrides(&mut self, args: &[String]) {
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            match arg.as_str() {
                "-d" | "--dump" => self.dump_frames = true,
                "-k" | "--kiss" => self.parse_kiss = true,
                "-a" | "--ax25" => self.dump_ax25 = true,
                "-q" | "--quiet" => self.quiet_startup = true,
                "-4" => { self.ipv4_only = true; self.ipv6_only = false; }
                "-6" => { self.ipv6_only = true; self.ipv4_only = false; }
                "-n" | "--phil" => self.phil_flag = true,
                "-R" | "--raw-copy" => self.raw_copy = true,
                "-x" | "--xon-xoff" => self.flow_control = FlowControl::Software,
                "-H" | "--rts-cts" => self.flow_control = FlowControl::Hardware,
                "--dtr-dsr" => self.flow_control = FlowControl::DtrDsr,
                "-N" | "--none" => self.flow_control = FlowControl::None,
                "--no-console" => { self.log_to_console = false; self.log_to_file_only = true; }
                "--console-only" => { self.log_to_console = true; self.logfile = None; }
                "-s" | "--stop-bits" => {
                    if i + 1 < args.len() {
                        self.stop_bits = match args[i + 1].as_str() {
                            "1" | "one" => StopBits::One,
                            "2" | "two" => StopBits::Two,
                            _ => self.stop_bits,
                        };
                        i += 1;
                    }
                }
                "-Q" | "--parity" => {
                    if i + 1 < args.len() {
                        self.parity = match args[i + 1].to_lowercase().as_str() {
                            "none" | "n" | "no" => Parity::None,
                            "odd" | "o" => Parity::Odd,
                            "even" | "e" => Parity::Even,
                            _ => self.parity,
                        };
                        i += 1;
                    }
                }
                "-l" | "--logfile" => {
                    if i + 1 < args.len() { self.logfile = Some(args[i + 1].clone()); i += 1; }
                }
                "-P" | "--pidfile" => {
                    if i + 1 < args.len() { self.pidfile = Some(args[i + 1].clone()); i += 1; }
                }
                "-p" | "--port" => {
                    if i + 1 < args.len() {
                        if let Ok(port) = args[i + 1].parse::<u16>() { self.tcp_port = port; }
                        i += 1;
                    }
                }
                "-b" | "--baud-rate" => {
                    if i + 1 < args.len() {
                        if let Ok(baud) = args[i + 1].parse::<u32>() { self.baud_rate = baud; }
                        i += 1;
                    }
                }
                "-D" | "--device" => {
                    if i + 1 < args.len() { self.serial_port = args[i + 1].clone(); i += 1; }
                }
                "-I" | "--address" => {
                    if i + 1 < args.len() {
                        self.tcp_addresses = args[i + 1]
                            .split(|c| c == ',' || c == ' ')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        i += 1;
                    }
                }
                "-L" | "--log-level" => {
                    if i + 1 < args.len() {
                        if let Ok(level) = args[i + 1].parse::<u8>() { self.log_level = level.min(9); }
                        i += 1;
                    }
                }
                "-c" => { if i + 1 < args.len() { i += 1; } }
                "--pcap" => {
                    if i + 1 < args.len() { self.pcap_file = Some(args[i + 1].clone()); i += 1; }
                }
                _ => {}
            }
            i += 1;
        }
        
        if self.ipv4_only {
            self.tcp_addresses.retain(|addr| !addr.contains(':') || addr.starts_with("::ffff:"));
        } else if self.ipv6_only {
            self.tcp_addresses.retain(|addr| addr.contains(':'));
        }
    }
}
// Part 2 of 4 - Append this after Part 1

#[derive(Debug)]
struct AX25Address {
    callsign: String,
    ssid: u8,
}

impl AX25Address {
    fn from_ax25_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 7 { return None; }
        let mut callsign = String::new();
        for i in 0..6 {
            let ch = (bytes[i] >> 1) as char;
            if ch != ' ' { callsign.push(ch); }
        }
        let ssid = (bytes[6] >> 1) & 0x0F;
        Some(AX25Address { callsign, ssid })
    }
    
    fn to_string(&self) -> String {
        if self.ssid == 0 { self.callsign.clone() } 
        else { format!("{}-{}", self.callsign, self.ssid) }
    }
}

#[derive(Debug)]
struct AX25Frame {
    destination: AX25Address,
    source: AX25Address,
    digipeaters: Vec<AX25Address>,
    control: u8,
    pid: Option<u8>,
    info: Vec<u8>,
}

#[derive(Debug, PartialEq)]
enum AX25FrameType {
    IFrame, SFrame, UFrame, UIFrame, Unknown,
}

impl AX25Frame {
    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 16 { return None; }
        let mut offset = 0;
        let destination = AX25Address::from_ax25_bytes(&data[offset..offset+7])?;
        offset += 7;
        let source = AX25Address::from_ax25_bytes(&data[offset..offset+7])?;
        offset += 7;
        let mut digipeaters = Vec::new();
        while offset + 7 <= data.len() {
            let addr_byte_6 = data[offset + 6];
            let digi = AX25Address::from_ax25_bytes(&data[offset..offset+7])?;
            digipeaters.push(digi);
            offset += 7;
            if addr_byte_6 & 0x01 != 0 { break; }
        }
        if offset >= data.len() { return None; }
        let control = data[offset];
        offset += 1;
        let pid = if (control & 0x03) == 0x00 || (control & 0x03) == 0x03 {
            if offset < data.len() { let p = data[offset]; offset += 1; Some(p) } else { None }
        } else { None };
        let info = data[offset..].to_vec();
        Some(AX25Frame { destination, source, digipeaters, control, pid, info })
    }
    
    fn get_frame_type(&self) -> AX25FrameType {
        if (self.control & 0x01) == 0 { AX25FrameType::IFrame }
        else if (self.control & 0x03) == 0x01 { AX25FrameType::SFrame }
        else if (self.control & 0x03) == 0x03 {
            if (self.control & 0xEF) == 0x03 { AX25FrameType::UIFrame } 
            else { AX25FrameType::UFrame }
        } else { AX25FrameType::Unknown }
    }
    
    fn get_connection_phase(&self) -> &str {
        match self.get_frame_type() {
            AX25FrameType::IFrame => "CONNECTED (Information Transfer)",
            AX25FrameType::SFrame => "CONNECTED (Supervisory)",
            AX25FrameType::UFrame => {
                match self.control & 0xEF {
                    0x2F => "SETUP (SABM)",
                    0x63 => "SETUP (SABME)",
                    0x43 => "DISCONNECT (DISC)",
                    0x0F => "DISCONNECT (DM)",
                    0x87 => "ERROR (FRMR)",
                    _ => "CONTROL (Unnumbered)",
                }
            }
            AX25FrameType::UIFrame => "UNCONNECTED (UI Frame)",
            AX25FrameType::Unknown => "UNKNOWN",
        }
    }
    
    fn print_summary(&self) {
        println!("  AX.25: {} > {}", self.source.to_string(), self.destination.to_string());
        if !self.digipeaters.is_empty() {
            print!("  Via: ");
            for (i, digi) in self.digipeaters.iter().enumerate() {
                if i > 0 { print!(", "); }
                print!("{}", digi.to_string());
            }
            println!();
        }
        println!("  Type: {:?}", self.get_frame_type());
        println!("  Phase: {}", self.get_connection_phase());
        println!("  Control: 0x{:02x}", self.control);
        if let Some(pid) = self.pid { println!("  PID: 0x{:02x}", pid); }
        if !self.info.is_empty() { println!("  Info: {} bytes", self.info.len()); }
    }
}

struct PcapWriter {
    file: Arc<Mutex<File>>,
}

impl PcapWriter {
    fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::create(path)?;
        file.write_all(&0xa1b2c3d4u32.to_le_bytes())?;
        file.write_all(&2u16.to_le_bytes())?;
        file.write_all(&4u16.to_le_bytes())?;
        file.write_all(&0i32.to_le_bytes())?;
        file.write_all(&0u32.to_le_bytes())?;
        file.write_all(&65535u32.to_le_bytes())?;
        file.write_all(&147u32.to_le_bytes())?;
        Ok(PcapWriter { file: Arc::new(Mutex::new(file)) })
    }
    
    fn write_packet(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
        let mut file = self.file.lock().unwrap();
        file.write_all(&(now.as_secs() as u32).to_le_bytes())?;
        file.write_all(&(now.subsec_micros() as u32).to_le_bytes())?;
        file.write_all(&(data.len() as u32).to_le_bytes())?;
        file.write_all(&(data.len() as u32).to_le_bytes())?;
        file.write_all(data)?;
        file.flush()?;
        Ok(())
    }
}

struct Logger {
    file: Option<Arc<Mutex<File>>>,
    log_level: u8,
    log_to_console: bool,
}

impl Logger {
    fn new(logfile: Option<String>, log_level: u8, log_to_console: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let file = if let Some(path) = logfile {
            let f = OpenOptions::new().create(true).append(true).open(path)?;
            Some(Arc::new(Mutex::new(f)))
        } else { None };
        Ok(Logger { file, log_level, log_to_console })
    }
    
    fn log(&self, message: &str, level: u8) {
        if level > self.log_level { return; }
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let level_str = match level {
            0 => "EMERG", 1 => "ALERT", 2 => "CRIT", 3 => "ERROR", 4 => "WARN",
            5 => "NOTICE", 6 => "INFO", 7 => "DEBUG", 8 => "TRACE", 9 => "VERBOSE",
            _ => "UNKNOWN",
        };
        let log_line = format!("[{}] [{}] {}\n", timestamp, level_str, message);
        if self.log_to_console { print!("{}", log_line); }
        if let Some(ref file) = self.file {
            if let Ok(mut f) = file.lock() {
                let _ = f.write_all(log_line.as_bytes());
            }
        }
    }
}

struct KissBridge {
    serial_port: Arc<Mutex<Box<dyn SerialPort>>>,
    config: Config,
    logger: Arc<Logger>,
    pcap_writer: Option<Arc<PcapWriter>>,
}

impl KissBridge {
    fn new(config: Config, logger: Arc<Logger>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut port_builder = serialport::new(&config.serial_port, config.baud_rate)
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
        
        let pcap_writer = if let Some(ref pcap_path) = config.pcap_file {
            Some(Arc::new(PcapWriter::new(pcap_path)?))
        } else { None };
        Ok(KissBridge { serial_port: Arc::new(Mutex::new(port)), config, logger, pcap_writer })
    }

    fn dump_frame(data: &[u8], title: &str) {
        println!("=== {} ({} bytes) ===", title, data.len());
        for (i, chunk) in data.chunks(16).enumerate() {
            print!("{:08x}: ", i * 16);
            for (j, byte) in chunk.iter().enumerate() {
                print!("{:02x}", byte);
                if j == 7 { print!(" "); }
                print!(" ");
            }
            if chunk.len() < 16 {
                for j in chunk.len()..16 {
                    if j == 8 { print!(" "); }
                    print!("   ");
                }
            }
            print!(" ");
            for byte in chunk {
                print!("{}", if *byte >= 0x20 && *byte <= 0x7e { *byte as char } else { '.' });
            }
            println!();
        }
        println!();
    }
    
    fn parse_kiss_frame(&self, data: &[u8], direction: &str) {
        if data.len() < 2 || data[0] != KISS_FEND { return; }
        let end_pos = match data.iter().skip(1).position(|&b| b == KISS_FEND) {
            Some(pos) => pos,
            None => return,
        };
        let frame_data = &data[1..end_pos + 1];
        if frame_data.is_empty() { return; }
        let cmd_byte = frame_data[0];
        let port = (cmd_byte >> 4) & 0x0F;
        let command = cmd_byte & 0x0F;
        let cmd_name = match command {
            0 => "Data", 1 => "TXDELAY", 2 => "Persistence", 3 => "SlotTime",
            4 => "TXtail", 5 => "FullDuplex", 6 => "SetHardware", 15 => "Return",
            _ => "Unknown",
        };
        
        let dir_header = if direction.contains("Serial") && direction.contains("TCP") {
            "TNC -> PC"
        } else if direction.contains("TCP") && direction.contains("Serial") {
            "PC -> TNC"
        } else {
            direction
        };
        
        println!("=== {} KISS Frame ===", dir_header);
        println!("  Port: {}, Command: {} ({})", port, command, cmd_name);
        println!("  Frame length: {} bytes", data.len());
        if command == 0 && frame_data.len() > 1 {
            let ax25_data = &frame_data[1..];
            if let Some(ref pcap) = self.pcap_writer {
                let _ = pcap.write_packet(ax25_data);
            }
            if let Some(ax25_frame) = AX25Frame::parse(ax25_data) {
                ax25_frame.print_summary();
                if self.config.dump_ax25 && !ax25_frame.info.is_empty() {
                    Self::dump_frame(&ax25_frame.info, &format!("{} AX.25 Info Field", dir_header));
                }
            }
        }
        println!();
    }
// Part 3 of 4 - Append this after Part 2

    fn handle_client(&self, mut stream: TcpStream) {
        self.logger.log(&format!("Client connected: {}", stream.peer_addr().unwrap()), 5);
        
        if self.config.raw_copy {
            self.logger.log("Raw copy mode enabled - transparent pass-through", 6);
            self.handle_raw_copy(stream);
            return;
        }
        
        let serial_clone = Arc::clone(&self.serial_port);
        let mut read_stream = stream.try_clone().expect("Failed to clone stream");
        let config = self.config.clone();
        let logger = Arc::clone(&self.logger);
        let pcap_writer = self.pcap_writer.clone();
        
        let serial_to_tcp = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            let mut frame_buffer = Vec::new();
            let mut in_frame = false;

            loop {
                let mut port = serial_clone.lock().unwrap();
                match port.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        if config.phil_flag {
                            for &byte in &buffer[..n] {
                                if byte == KISS_FEND {
                                    if in_frame {
                                        frame_buffer.push(byte);
                                        let processed = process_frame_with_phil_flag(&frame_buffer);
                                        if config.parse_kiss {
                                            KissBridge::parse_kiss_frame_static(&processed, "TNC -> PC", &config, &pcap_writer);
                                        } else if config.dump_frames {
                                            KissBridge::dump_frame(&processed, "TNC -> PC KISS Frame");
                                        }
                                        if let Err(e) = read_stream.write_all(&processed) {
                                            logger.log(&format!("Error writing to TCP: {}", e), 3);
                                            return;
                                        }
                                        frame_buffer.clear();
                                        in_frame = false;
                                    } else {
                                        frame_buffer.push(byte);
                                        in_frame = true;
                                    }
                                } else {
                                    frame_buffer.push(byte);
                                }
                            }
                        } else {
                            if config.parse_kiss {
                                KissBridge::parse_kiss_frame_static(&buffer[..n], "TNC -> PC", &config, &pcap_writer);
                            } else if config.dump_frames {
                                KissBridge::dump_frame(&buffer[..n], "TNC -> PC KISS Frame");
                            }
                            if let Err(e) = read_stream.write_all(&buffer[..n]) {
                                logger.log(&format!("Error writing to TCP: {}", e), 3);
                                break;
                            }
                        }
                    }
                    Ok(_) => thread::sleep(Duration::from_millis(10)),
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        logger.log(&format!("Error reading from serial: {}", e), 3);
                        break;
                    }
                }
            }
        });

        let mut buffer = [0u8; 1024];
        let serial_clone = Arc::clone(&self.serial_port);
        
        loop {
            match stream.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    if self.config.parse_kiss {
                        self.parse_kiss_frame(&buffer[..n], "PC -> TNC");
                    } else if self.config.dump_frames {
                        Self::dump_frame(&buffer[..n], "PC -> TNC KISS Frame");
                    }
                    
                    let data_to_send = if self.config.phil_flag {
                        process_phil_flag_tcp_to_serial(&buffer[..n])
                    } else {
                        buffer[..n].to_vec()
                    };
                    
                    let mut port = serial_clone.lock().unwrap();
                    if let Err(e) = port.write_all(&data_to_send) {
                        self.logger.log(&format!("Error writing to serial: {}", e), 3);
                        break;
                    }
                }
                Ok(_) => { self.logger.log("Client disconnected", 5); break; }
                Err(e) => { self.logger.log(&format!("Error reading from TCP: {}", e), 3); break; }
            }
        }
        drop(serial_to_tcp);
        self.logger.log("Connection closed", 5);
    }
    
    fn handle_raw_copy(&self, mut stream: TcpStream) {
        let serial_clone = Arc::clone(&self.serial_port);
        let mut read_stream = stream.try_clone().expect("Failed to clone stream");
        let logger = Arc::clone(&self.logger);
        
        let serial_to_tcp = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                let mut port = serial_clone.lock().unwrap();
                match port.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        if let Err(e) = read_stream.write_all(&buffer[..n]) {
                            logger.log(&format!("Raw copy: Error writing to TCP: {}", e), 3);
                            break;
                        }
                    }
                    Ok(_) => thread::sleep(Duration::from_millis(10)),
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        logger.log(&format!("Raw copy: Error reading from serial: {}", e), 3);
                        break;
                    }
                }
            }
        });

        let mut buffer = [0u8; 1024];
        let serial_clone = Arc::clone(&self.serial_port);
        
        loop {
            match stream.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let mut port = serial_clone.lock().unwrap();
                    if let Err(e) = port.write_all(&buffer[..n]) {
                        self.logger.log(&format!("Raw copy: Error writing to serial: {}", e), 3);
                        break;
                    }
                }
                Ok(_) => {
                    self.logger.log("Raw copy: Client disconnected", 5);
                    break;
                }
                Err(e) => {
                    self.logger.log(&format!("Raw copy: Error reading from TCP: {}", e), 3);
                    break;
                }
            }
        }
        drop(serial_to_tcp);
        self.logger.log("Raw copy: Connection closed", 5);
    }
    
    fn parse_kiss_frame_static(data: &[u8], direction: &str, config: &Config, pcap_writer: &Option<Arc<PcapWriter>>) {
        if data.len() < 2 || data[0] != KISS_FEND { return; }
        let end_pos = match data.iter().skip(1).position(|&b| b == KISS_FEND) {
            Some(pos) => pos,
            None => return,
        };
        let frame_data = &data[1..end_pos + 1];
        if frame_data.is_empty() { return; }
        let cmd_byte = frame_data[0];
        let port = (cmd_byte >> 4) & 0x0F;
        let command = cmd_byte & 0x0F;
        let cmd_name = match command {
            0 => "Data", 1 => "TXDELAY", 2 => "Persistence", 3 => "SlotTime",
            4 => "TXtail", 5 => "FullDuplex", 6 => "SetHardware", 15 => "Return",
            _ => "Unknown",
        };
        
        let dir_header = if direction.contains("Serial") && direction.contains("TCP") {
            "TNC -> PC"
        } else if direction.contains("TCP") && direction.contains("Serial") {
            "PC -> TNC"
        } else {
            direction
        };
        
        println!("=== {} KISS Frame ===", dir_header);
        println!("  Port: {}, Command: {} ({})", port, command, cmd_name);
        println!("  Frame length: {} bytes", data.len());
        if command == 0 && frame_data.len() > 1 {
            let ax25_data = &frame_data[1..];
            if let Some(ref pcap) = pcap_writer {
                let _ = pcap.write_packet(ax25_data);
            }
            if let Some(ax25_frame) = AX25Frame::parse(ax25_data) {
                ax25_frame.print_summary();
                if config.dump_ax25 && !ax25_frame.info.is_empty() {
                    KissBridge::dump_frame(&ax25_frame.info, &format!("{} AX.25 Info Field", dir_header));
                }
            }
        }
        println!();
    }

    fn start_server(&self, addresses: &[String], port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let mut listeners = Vec::new();
        for address in addresses {
            let bind_address = format!("{}:{}", address, port);
            match TcpListener::bind(&bind_address) {
                Ok(listener) => {
                    self.logger.log(&format!("KISS-over-TCP server listening on {}", bind_address), 5);
                    listeners.push(listener);
                }
                Err(e) => {
                    self.logger.log(&format!("Warning: Failed to bind to {}: {}", bind_address, e), 4);
                }
            }
        }
        if listeners.is_empty() { return Err("Failed to bind to any address".into()); }
        self.logger.log(&format!("PhilFlag: {}", if self.config.phil_flag { "ENABLED" } else { "DISABLED" }), 5);

        loop {
            for listener in &listeners {
                if let Ok(listener_clone) = listener.try_clone() {
                    if listener_clone.set_nonblocking(true).is_ok() {
                        if let Ok((stream, _)) = listener.accept() {
                            let _ = stream.set_nonblocking(false);
                            self.handle_client(stream);
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

// In for drought? No, we are in for turbo-hydrometeorology!
//
// PhilFlag Processing Functions

fn process_frame_with_phil_flag(frame: &[u8]) -> Vec<u8> {
    if frame.len() < 2 { return frame.to_vec(); }
    let mut output = Vec::with_capacity(frame.len() * 2);
    output.push(frame[0]);
    for i in 1..frame.len()-1 {
        if frame[i] == KISS_FEND {
            output.push(KISS_FESC);
            output.push(KISS_TFEND);
        } else {
            output.push(frame[i]);
        }
    }
    if frame.len() > 1 { output.push(frame[frame.len()-1]); }
    output
}

fn process_phil_flag_tcp_to_serial(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(data.len() * 2);
    for &byte in data {
        if byte == 0x43 || byte == 0x63 {
            output.push(KISS_FESC);
            output.push(byte);
        } else {
            output.push(byte);
        }
    }
    output
}

fn write_pidfile(pidfile: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(pidfile)?;
    writeln!(file, "{}", std::process::id())?;
    Ok(())
}
// Part 4 of 4 - Append this after Part 3

fn show_help(program_name: &str) {
    println!("rax25kb - AX.25 KISS Bridge\n");
    println!("Usage: {} [OPTIONS]\n", program_name);
    println!("Serial Port Options:");
    println!("  -D, --device <dev>    Serial port (Linux: /dev/ttyUSB0, Windows: COM3)");
    println!("  -b, --baud-rate <rate>  Baud rate (default: 9600)");
    println!("  -s, --stop-bits <1|2>  Stop bits (default: 1 for KISS TNC)");
    println!("  -Q, --parity <n|e|o>  Parity: none, even, odd (default: none for KISS TNC)");
    println!("\nFlow Control Options:");
    println!("  -x, --xon-xoff        Software flow control (XON/XOFF)");
    println!("  -H, --rts-cts         Hardware flow control (RTS/CTS)");
    println!("  --dtr-dsr             DTR/DSR flow control");
    println!("  -N, --none            No flow control (default)");
    println!("\nNetwork Options:");
    println!("  -I, --address <addr>  TCP address(es) - space/comma separated");
    println!("  -p, --port <port>     TCP port (default: 8001)");
    println!("  -4                    IPv4 only");
    println!("  -6                    IPv6 only");
    println!("\nFeature Options:");
    println!("  -d, --dump            Dump KISS frames in xxd format");
    println!("  -k, --kiss            Parse and display KISS frame info");
    println!("  -a, --ax25            Dump AX.25 info fields");
    println!("  -n, --phil            Enable PhilFlag (C0->DB DC, C/c->DB C/c correction)");
    println!("  -R, --raw-copy        Raw copy mode (transparent pass-through, no KISS)");
    println!("\nLogging Options:");
    println!("  -l, --logfile <file>  Log file path");
    println!("  -L, --log-level <0-9>  Log level (default: 5)");
    println!("  --console-only        Log only to console");
    println!("  --no-console          Log only to file");
    println!("\nOther Options:");
    println!("  -P, --pidfile <file>  PID file path");
    println!("  --pcap <file>         Write AX.25 frames to PCAP file");
    println!("  -c <file>             Config file (default: rax25kb.cfg)");
    println!("  -q, --quiet           Quiet startup");
    println!("  -h, --help            Show this help\n");
    println!("KISS TNC Defaults: 8N1 (8 data bits, No parity, 1 stop bit), No flow control");
    println!("PhilFlag: Serial->TCP (C0->DB DC), TCP->Serial (C/c->DB C/c to prevent TC0\\n)");
    println!("Raw Copy: Disables KISS, PhilFlag, parsing, PCAP - direct serial/TCP bridge\n");
    println!("Config file options:");
    println!("  serial_port, baud_rate, stop_bits (1|2), parity (none|even|odd)");
    println!("  flow_control (none|software|hardware|dtrdsr)");
    println!("  tcp_address, tcp_port, phil_flag, dump, parse_kiss, dump_ax25");
    println!("  raw_copy, log_level, logfile, pidfile, pcap_file\n");
    println!("Examples:");
    println!("  {} -D /dev/ttyUSB0 -b 9600 -k        # Parse KISS frames", program_name);
    println!("  {} -D /dev/ttyUSB0 -n -k --pcap f.pcap  # With PhilFlag & capture", program_name);
    println!("  {} -D COM3 -R                        # Windows raw mode", program_name);
    println!("  {} -I \"0.0.0.0 ::\" -p 8001          # IPv4+IPv6 listener", program_name);
    println!("\nLog levels: 0=EMERG 1=ALERT 2=CRIT 3=ERROR 4=WARN 5=NOTICE 6=INFO 7=DEBUG 8=TRACE 9=VERBOSE");
    println!("\nFor more information, see: https://www.outpostpm.org/support.html");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\nReceived SIGINT, shutting down gracefully...");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
        std::process::exit(0);
    })?;
    
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        show_help(&args[0]);
        return Ok(());
    }
    
    let config_file = args.iter()
        .position(|arg| arg == "-c")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("rax25kb.cfg");
    
    let mut config = Config::from_file(config_file)?;
    config.apply_cli_overrides(&args);
    
    if !config.quiet_startup {
        println!("rax25kb - AX.25 KISS Bridge");
        println!("==========================");
        println!("Configuration from: {}", config_file);
        println!("  Serial: {} @ {} baud", config.serial_port, config.baud_rate);
        let flow_str = match config.flow_control {
            FlowControl::None => "None",
            FlowControl::Software => "Software (XON/XOFF)",
            FlowControl::Hardware => "Hardware (RTS/CTS)",
            FlowControl::DtrDsr => "DTR/DSR",
        };
        println!("  Flow control: {}", flow_str);
        let stop_str = match config.stop_bits {
            StopBits::One => "1",
            StopBits::Two => "2",
        };
        let parity_str = match config.parity {
            Parity::None => "None",
            Parity::Odd => "Odd",
            Parity::Even => "Even",
        };
        println!("  Format: 8{}{} (8 data bits, {} parity, {} stop bit{})",
                 parity_str.chars().next().unwrap(),
                 stop_str,
                 parity_str,
                 stop_str,
                 if config.stop_bits == StopBits::One { "" } else { "s" });
        println!("  TCP: {} port {}", config.tcp_addresses.join(", "), config.tcp_port);
        println!("  PhilFlag: {}", if config.phil_flag { "ON" } else { "OFF" });
        println!("  Raw copy: {}", if config.raw_copy { "ON" } else { "OFF" });
        println!("  Log level: {}", config.log_level);
        if let Some(ref lf) = config.logfile { println!("  Log file: {}", lf); }
        if let Some(ref pf) = config.pidfile { println!("  PID file: {}", pf); }
        if let Some(ref pcap) = config.pcap_file { println!("  PCAP file: {}", pcap); }
        println!();
    }
    
    if let Some(ref pidfile) = config.pidfile {
        write_pidfile(pidfile)?;
        if !config.quiet_startup {
            println!("PID {} written to {}", std::process::id(), pidfile);
        }
    }
    
    let logger = Arc::new(Logger::new(config.logfile.clone(), config.log_level, config.log_to_console)?);
    logger.log("rax25kb starting", 5);

    let bridge = KissBridge::new(config.clone(), logger)?;
    bridge.start_server(&config.tcp_addresses, config.tcp_port)?;
    Ok(())
}
