// rax25kb - AX.25 KISS Bridge with Multi-Port Cross-Connect Support
// Version 1.6.3
//
// Copyright (C) 2025 Kris Kirby, KE4AHR
//
// SPDX-License-Identifier: GPL-3.0-or-later
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
// ==============================================================================
// MAIN.RS - PART 1 OF 5
// ==============================================================================
//
// This part contains:
// - Standard library imports
// - External crate dependencies
// - KISS protocol constants
// - Configuration data structures
// - AX.25 protocol structures and parsing
//
// Features supported:
// - Multiple serial ports with independent configurations
// - Cross-connects between serial ports and TCP sockets
// - Serial-to-serial bridging with KISS port translation
// - Standard KISS to Extended KISS translation
// - Per-connection feature flags (PhilFlag, raw copy, parsing)
//
// ==============================================================================

// ==============================================================================
// STANDARD LIBRARY IMPORTS
// ==============================================================================

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::process;

// ==============================================================================
// KISS PROTOCOL CONSTANTS
// ==============================================================================

const KISS_FEND: u8 = 0xC0;
const KISS_FESC: u8 = 0xDB;
const KISS_TFEND: u8 = 0xDC;
#[allow(dead_code)]
const KISS_TFESC: u8 = 0xDD;

// ==============================================================================
// CONFIGURATION STRUCTURES
// ==============================================================================

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
        kiss_port: u8,
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
    extended_kiss: bool,
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

// ==============================================================================
// AX.25 PROTOCOL STRUCTURES
// ==============================================================================

#[derive(Debug)]
struct AX25Address {
    callsign: String,
    ssid: u8,
}

impl AX25Address {
    fn from_ax25_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 7 { 
            return None; 
        }
        
        let mut callsign = String::new();
        for i in 0..6 {
            let ch = (bytes[i] >> 1) as char;
            if ch != ' ' {
                callsign.push(ch); 
            }
        }
        
        let ssid = (bytes[6] >> 1) & 0x0F;
        
        Some(AX25Address { callsign, ssid })
    }
    
    fn to_string(&self) -> String {
        if self.ssid == 0 { 
            self.callsign.clone() 
        } else { 
            format!("{}-{}", self.callsign, self.ssid) 
        }
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
    IFrame,
    SFrame,
    UFrame,
    UIFrame,
    Unknown,
}

impl AX25Frame {
    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 16 { 
            return None; 
        }
        
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
            
            if addr_byte_6 & 0x01 != 0 { 
                break; 
            }
        }
        
        if offset >= data.len() { 
            return None; 
        }
        
        let control = data[offset];
        offset += 1;
        
        let pid = if (control & 0x03) == 0x00 || (control & 0x03) == 0x03 {
            if offset < data.len() { 
                let p = data[offset]; 
                offset += 1; 
                Some(p) 
            } else { 
                None 
            }
        } else { 
            None 
        };
        
        let info = data[offset..].to_vec();
        
        Some(AX25Frame { 
            destination, 
            source, 
            digipeaters, 
            control, 
            pid, 
            info 
        })
    }
    
    fn get_frame_type(&self) -> AX25FrameType {
        if (self.control & 0x01) == 0 {
            AX25FrameType::IFrame 
        } else if (self.control & 0x03) == 0x01 {
            AX25FrameType::SFrame 
        } else if (self.control & 0x03) == 0x03 {
            if (self.control & 0xEF) == 0x03 { 
                AX25FrameType::UIFrame 
            } else { 
                AX25FrameType::UFrame 
            }
        } else { 
            AX25FrameType::Unknown 
        }
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
        println!("  AX.25: {} > {}", 
            self.source.to_string(), 
            self.destination.to_string()
        );
        
        if !self.digipeaters.is_empty() {
            print!("  Via: ");
            for (i, digi) in self.digipeaters.iter().enumerate() {
                if i > 0 { 
                    print!(", "); 
                }
                print!("{}", digi.to_string());
            }
            println!();
        }
        
        println!("  Type: {:?}", self.get_frame_type());
        println!("  Phase: {}", self.get_connection_phase());
        println!("  Control: 0x{:02x}", self.control);
        
        if let Some(pid) = self.pid { 
            println!("  PID: 0x{:02x}", pid); 
        }
        
        if !self.info.is_empty() { 
            println!("  Info: {} bytes", self.info.len()); 
        }
    }
}
// ==============================================================================
// MAIN.RS - PART 2 OF 5
// ==============================================================================
//
// This part contains:
// - Configuration file parsing implementation
// - Serial port configuration parsing
// - Cross-connect configuration parsing
// - Endpoint parsing and validation
// - Helper functions for parsing enums and booleans
//
// ==============================================================================

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
        
        let mut serial_ports = HashMap::new();
        let mut serial_port_ids = Vec::new();
        
        for key in config_map.keys() {
            if key.starts_with("serial_port") && key.len() > 11 {
                let id = &key[11..];
                
                if id.chars().all(|c| c.is_ascii_digit()) && id.len() == 4 {
                    if !serial_port_ids.contains(&id.to_string()) {
                        serial_port_ids.push(id.to_string());
                    }
                }
            }
        }
        
        for id in &serial_port_ids {
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
            
            serial_ports.insert(id.clone(), port_config);
        }
        
        let mut cross_connects = Vec::new();
        let mut cross_connect_ids = Vec::new();
        
        for key in config_map.keys() {
            if key.starts_with("cross_connect") && key.len() == 17 {
                let id = &key[13..17];
                
                if id.chars().all(|c| c.is_ascii_digit()) {
                    if !cross_connect_ids.contains(&id.to_string()) {
                        cross_connect_ids.push(id.to_string());
                    }
                }
            }
        }
        
        cross_connect_ids.sort();
        
        for id in cross_connect_ids {
            let cc_key = format!("cross_connect{}", id);
            let cc_value = config_map.get(&cc_key)
                .ok_or(format!("Missing cross_connect{}", id))?;
            
            let parts: Vec<&str> = cc_value.split("<->").collect();
            if parts.len() != 2 {
                return Err(format!(
                    "Invalid cross_connect{} format: {} (expected: endpoint <-> endpoint)", 
                    id, cc_value
                ).into());
            }
            
            let endpoint_a = Self::parse_endpoint(parts[0].trim(), &serial_ports)?;
            let endpoint_b = Self::parse_endpoint(parts[1].trim(), &serial_ports)?;
            
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
        
        if cross_connects.is_empty() && !serial_ports.is_empty() {
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
    
    fn parse_endpoint(
        s: &str, 
        serial_ports: &HashMap<String, SerialPortConfig>
    ) -> Result<CrossConnectEndpoint, Box<dyn std::error::Error>> {
        
        let parts: Vec<&str> = s.split(':').collect();
        
        if parts.is_empty() {
            return Err("Empty endpoint specification".into());
        }
        
        match parts[0] {
            "tcp" => {
                if parts.len() != 3 {
                    return Err(format!(
                        "Invalid TCP endpoint format: {} (expected: tcp:address:port)", 
                        s
                    ).into());
                }
                
                let address = parts[1].to_string();
                let port = parts[2].parse::<u16>()
                    .map_err(|_| format!("Invalid TCP port: {}", parts[2]))?;
                
                Ok(CrossConnectEndpoint::TcpSocket { address, port })
            }
            
            "serial" => {
                if parts.len() != 3 {
                    return Err(format!(
                        "Invalid serial endpoint format: {} (expected: serial:port_id:kiss_port)", 
                        s
                    ).into());
                }
                
                let port_id = parts[1].to_string();
                let kiss_port = parts[2].parse::<u8>()
                    .map_err(|_| format!("Invalid KISS port: {}", parts[2]))?;
                
                if kiss_port > 15 {
                    return Err(format!(
                        "KISS port must be 0-15, got: {}", 
                        kiss_port
                    ).into());
                }
                
                if !serial_ports.contains_key(&port_id) {
                    return Err(format!(
                        "Unknown serial port ID: {} (not defined in serial_port{} entries)", 
                        port_id, port_id
                    ).into());
                }
                
                Ok(CrossConnectEndpoint::SerialPort { port_id, kiss_port })
            }
            
            _ => {
                Err(format!(
                    "Invalid endpoint type: {} (expected: 'tcp' or 'serial')", 
                    parts[0]
                ).into())
            }
        }
    }
    
    fn parse_flow_control(s: &str) -> Option<FlowControl> {
        match s.to_lowercase().as_str() {
            "software" | "xon" | "xonxoff" | "xon-xoff" => {
                Some(FlowControl::Software)
            }
            "hardware" | "rtscts" | "rts-cts" | "rts/cts" => {
                Some(FlowControl::Hardware)
            }
            "dtrdsr" | "dtr-dsr" | "dtr/dsr" => {
                Some(FlowControl::DtrDsr)
            }
            "none" | "off" | "no" => {
                Some(FlowControl::None)
            }
            _ => None,
        }
    }
    
    fn parse_stop_bits(s: &str) -> Option<StopBits> {
        match s {
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
// ==============================================================================
// MAIN.RS - PART 3 OF 5
// ==============================================================================
//
// This part contains:
// - Logger implementation
// - PCAP writer
// - KISS frame buffer
// - KISS helper functions
// - KISS port translator
// - PhilFlag processing
// - Frame parsing and display
//
// ==============================================================================

struct Logger {
    file: Option<Arc<Mutex<File>>>,
    log_level: u8,
    log_to_console: bool,
}

impl Logger {
    fn new(
        logfile: Option<String>, 
        log_level: u8, 
        log_to_console: bool
    ) -> Result<Self, Box<dyn std::error::Error>> {
        
        let file = if let Some(path) = logfile {
            let f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            Some(Arc::new(Mutex::new(f)))
        } else { 
            None 
        };
        
        Ok(Logger { 
            file, 
            log_level, 
            log_to_console 
        })
    }
    
    fn log(&self, message: &str, level: u8) {
        if level > self.log_level { 
            return; 
        }
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        
        let level_str = match level {
            0 => "EMERG",
            1 => "ALERT",
            2 => "CRIT",
            3 => "ERROR",
            4 => "WARN",
            5 => "NOTICE",
            6 => "INFO",
            7 => "DEBUG",
            8 => "TRACE",
            9 => "VERBOSE",
            _ => "UNKNOWN",
        };
        
        let log_line = format!("[{}] [{}] {}\n", timestamp, level_str, message);
        
        if self.log_to_console { 
            print!("{}", log_line); 
        }
        
        if let Some(ref file) = self.file {
            if let Ok(mut f) = file.lock() {
                let _ = f.write_all(log_line.as_bytes());
            }
        }
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
        
        Ok(PcapWriter { 
            file: Arc::new(Mutex::new(file)) 
        })
    }
    
    fn write_packet(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?;
        
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

struct KissFrameBuffer {
    buffer: Vec<u8>,
    in_frame: bool,
}

impl KissFrameBuffer {
    fn new() -> Self {
        KissFrameBuffer {
            buffer: Vec::new(),
            in_frame: false,
        }
    }
    
    fn add_bytes(&mut self, data: &[u8]) -> Vec<Vec<u8>> {
        let mut frames = Vec::new();
        
        for &byte in data {
            if byte == KISS_FEND {
                if self.in_frame && !self.buffer.is_empty() {
                    self.buffer.push(byte);
                    frames.push(self.buffer.clone());
                    self.buffer.clear();
                    self.in_frame = false;
                } else {
                    self.buffer.clear();
                    self.buffer.push(byte);
                    self.in_frame = true;
                }
            } else if self.in_frame {
                self.buffer.push(byte);
            }
        }
        
        frames
    }
}

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

struct KissPortTranslator {
    source_port: u8,
    dest_port: u8,
}

impl KissPortTranslator {
    fn new(source_port: u8, dest_port: u8) -> Self {
        KissPortTranslator { 
            source_port, 
            dest_port 
        }
    }
    
    fn translate(&self, frame: &[u8]) -> Option<Vec<u8>> {
        let (current_port, _command, _data_start) = extract_kiss_info(frame)?;
        
        if current_port != self.source_port {
            return None;
        }
        
        if self.source_port == self.dest_port {
            return None;
        }
        
        Some(modify_kiss_port(frame, self.dest_port))
    }
}

fn process_frame_with_phil_flag(frame: &[u8]) -> Vec<u8> {
    if frame.len() < 2 { 
        return frame.to_vec(); 
    }
    
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
    
    if frame.len() > 1 { 
        output.push(frame[frame.len()-1]); 
    }
    
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

fn parse_kiss_frame_static(
    data: &[u8], 
    direction: &str, 
    pcap_writer: &Option<Arc<PcapWriter>>,
    dump_ax25: bool
) {
    if data.len() < 2 || data[0] != KISS_FEND { 
        return; 
    }
    
    let end_pos = match data.iter().skip(1).position(|&b| b == KISS_FEND) {
        Some(pos) => pos + 1,
        None => return,
    };
    
    let frame_data = &data[1..end_pos];
    if frame_data.is_empty() { 
        return; 
    }
    
    let cmd_byte = frame_data[0];
    let port = (cmd_byte >> 4) & 0x0F;
    let command = cmd_byte & 0x0F;
    
    let cmd_name = match command {
        0 => "Data",
        1 => "TXDELAY",
        2 => "Persistence",
        3 => "SlotTime",
        4 => "TXtail",
        5 => "FullDuplex",
        6 => "SetHardware",
        15 => "Return",
        _ => "Unknown",
    };
    
    println!("=== {} KISS Frame ===", direction);
    println!("  Port: {}, Command: {} ({})", port, command, cmd_name);
    println!("  Frame length: {} bytes", data.len());
    
    if command == 0 && frame_data.len() > 1 {
        let ax25_data = &frame_data[1..];
        
        if let Some(ref pcap) = pcap_writer {
            let _ = pcap.write_packet(ax25_data);
        }
        
        if dump_ax25 {
            if let Some(ax25_frame) = AX25Frame::parse(ax25_data) {
                ax25_frame.print_summary();
            }
        }
    }
    
    println!();
}

fn dump_frame(data: &[u8], title: &str) {
    println!("=== {} ({} bytes) ===", title, data.len());
    
    for (i, chunk) in data.chunks(16).enumerate() {
        print!("{:08x}: ", i * 16);
        
        for (j, byte) in chunk.iter().enumerate() {
            print!("{:02x}", byte);
            if j == 7 { 
                print!(" "); 
            }
            print!(" ");
        }
        
        if chunk.len() < 16 {
            for j in chunk.len()..16 {
                if j == 8 { 
                    print!(" "); 
                }
                print!("   ");
            }
        }
        
        print!(" ");
        
        for byte in chunk {
            let ch = if *byte >= 0x20 && *byte <= 0x7e { 
                *byte as char 
            } else { 
                '.' 
            };
            print!("{}", ch);
        }
        
        println!();
    }
    
    println!();
}
// ==============================================================================
// MAIN.RS - PART 4 OF 5
// ==============================================================================
//
// This part contains:
// - SerialPortManager
// - CrossConnectManager
// - Connection handlers
// - XKISS polled mode support
//
// ==============================================================================

use std::collections::VecDeque;

/// Queue for storing frames in XKISS polled mode
struct PolledModeQueue {
    frames: Arc<Mutex<VecDeque<Vec<u8>>>>,
    max_size: usize,
}

impl PolledModeQueue {
    fn new(max_size: usize) -> Self {
        PolledModeQueue {
            frames: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
        }
    }
    
    fn push(&self, frame: Vec<u8>) {
        let mut queue = self.frames.lock().unwrap();
        if queue.len() < self.max_size {
            queue.push_back(frame);
        }
        // Drop frame if queue is full
    }
    
    fn pop(&self) -> Option<Vec<u8>> {
        let mut queue = self.frames.lock().unwrap();
        queue.pop_front()
    }
    
    fn is_empty(&self) -> bool {
        let queue = self.frames.lock().unwrap();
        queue.is_empty()
    }
    
    fn clone_arc(&self) -> Arc<Mutex<VecDeque<Vec<u8>>>> {
        Arc::clone(&self.frames)
    }
}

struct SerialPortManager {
    ports: HashMap<String, Arc<Mutex<Box<dyn serialport::SerialPort>>>>,
}

impl SerialPortManager {
    fn new() -> Self {
        SerialPortManager { 
            ports: HashMap::new() 
        }
    }
    
    fn open_port(
        &mut self, 
        config: &SerialPortConfig
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        let mut port_builder = serialport::new(&config.device, config.baud_rate)
            .timeout(Duration::from_millis(100));
        
        port_builder = match config.flow_control {
            FlowControl::None => {
                port_builder.flow_control(serialport::FlowControl::None)
            }
            FlowControl::Software => {
                port_builder.flow_control(serialport::FlowControl::Software)
            }
            FlowControl::Hardware => {
                port_builder.flow_control(serialport::FlowControl::Hardware)
            }
            FlowControl::DtrDsr => {
                #[cfg(target_os = "windows")]
                { 
                    port_builder.flow_control(serialport::FlowControl::Hardware) 
                }
                #[cfg(not(target_os = "windows"))]
                { 
                    port_builder.flow_control(serialport::FlowControl::None) 
                }
            }
        };
        
        port_builder = match config.stop_bits {
            StopBits::One => {
                port_builder.stop_bits(serialport::StopBits::One)
            }
            StopBits::Two => {
                port_builder.stop_bits(serialport::StopBits::Two)
            }
        };
        
        port_builder = match config.parity {
            Parity::None => {
                port_builder.parity(serialport::Parity::None)
            }
            Parity::Odd => {
                port_builder.parity(serialport::Parity::Odd)
            }
            Parity::Even => {
                port_builder.parity(serialport::Parity::Even)
            }
        };
        
        let port = port_builder.open()?;
        
        self.ports.insert(
            config.id.clone(), 
            Arc::new(Mutex::new(port))
        );
        
        Ok(())
    }
    
    fn get_port(&self, id: &str) -> Option<Arc<Mutex<Box<dyn serialport::SerialPort>>>> {
        self.ports.get(id).map(|p| Arc::clone(p))
    }
}

struct CrossConnectManager {
    config: Arc<Config>,
    serial_manager: Arc<Mutex<SerialPortManager>>,
    logger: Arc<Logger>,
    pcap_writer: Option<Arc<PcapWriter>>,
}

impl CrossConnectManager {
    fn new(
        config: Config, 
        logger: Arc<Logger>, 
        pcap_writer: Option<Arc<PcapWriter>>
    ) -> Result<Self, Box<dyn std::error::Error>> {
        
        let mut serial_manager = SerialPortManager::new();
        
        for (id, port_config) in &config.serial_ports {
            logger.log(
                &format!("Opening serial port {}: {}", id, port_config.device), 
                5
            );
            serial_manager.open_port(port_config)?;
        }
        
        Ok(CrossConnectManager {
            config: Arc::new(config),
            serial_manager: Arc::new(Mutex::new(serial_manager)),
            logger,
            pcap_writer,
        })
    }
    
    fn start_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for cc in &self.config.cross_connects {
            self.logger.log(
                &format!("Starting cross-connect {}", cc.id), 
                5
            );
            self.start_cross_connect(cc)?;
        }
        Ok(())
    }
    
    fn start_cross_connect(
        &self, 
        cc: &CrossConnect
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        match (&cc.endpoint_a, &cc.endpoint_b) {
            (CrossConnectEndpoint::SerialPort { port_id, kiss_port }, 
             CrossConnectEndpoint::TcpSocket { address, port }) |
            (CrossConnectEndpoint::TcpSocket { address, port },
             CrossConnectEndpoint::SerialPort { port_id, kiss_port }) => {
                self.start_serial_to_tcp(cc, port_id, *kiss_port, address, *port)?;
            }
            
            (CrossConnectEndpoint::SerialPort { port_id: id_a, kiss_port: port_a },
             CrossConnectEndpoint::SerialPort { port_id: id_b, kiss_port: port_b }) => {
                self.start_serial_to_serial(cc, id_a, *port_a, id_b, *port_b)?;
            }
            
            (CrossConnectEndpoint::TcpSocket { .. }, 
             CrossConnectEndpoint::TcpSocket { .. }) => {
                return Err("TCP to TCP cross-connects not supported".into());
            }
        }
        
        Ok(())
    }
    
    fn start_serial_to_tcp(
        &self, 
        cc: &CrossConnect, 
        serial_id: &str, 
        kiss_port: u8,
        tcp_address: &str, 
        tcp_port: u16
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        let bind_address = format!("{}:{}", tcp_address, tcp_port);
        let listener = TcpListener::bind(&bind_address)?;
        
        self.logger.log(
            &format!("Cross-connect {} listening on {}", cc.id, bind_address), 
            5
        );
        
        let serial_manager = Arc::clone(&self.serial_manager);
        let serial_id = serial_id.to_string();
        let cc_config = cc.clone();
        let logger = Arc::clone(&self.logger);
        let pcap_writer = self.pcap_writer.clone();
        let config = Arc::clone(&self.config);
        
        thread::spawn(move || {
            loop {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        logger.log(
                            &format!("Cross-connect {}: Client connected from {}", 
                                cc_config.id, addr), 
                            5
                        );
                        
                        let serial_port = {
                            let mgr = serial_manager.lock().unwrap();
                            mgr.get_port(&serial_id)
                        };
                        
                        let port_config = config.serial_ports.get(&serial_id).cloned();
                        
                        if let (Some(port), Some(cfg)) = (serial_port, port_config) {
                            Self::handle_serial_tcp(
                                stream, 
                                port, 
                                kiss_port, 
                                &cc_config, 
                                &logger, 
                                &pcap_writer,
                                &cfg,
                            );
                        } else {
                            logger.log(
                                &format!("Serial port {} not found", serial_id), 
                                3
                            );
                        }
                    }
                    Err(e) => {
                        logger.log(
                            &format!("Accept error: {}", e), 
                            3
                        );
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        });
        
        Ok(())
    }
    
    fn handle_serial_tcp(
        mut stream: TcpStream, 
        serial_port: Arc<Mutex<Box<dyn serialport::SerialPort>>>,
        kiss_port: u8, 
        cc_config: &CrossConnect, 
        logger: &Arc<Logger>,
        pcap_writer: &Option<Arc<PcapWriter>>,
        port_config: &SerialPortConfig,
    ) {
        if cc_config.raw_copy {
            Self::handle_raw_copy(stream, serial_port, logger);
            return;
        }
        
        // Check if we need XKISS polled mode
        let polled_queue = if port_config.extended_kiss && port_config.polled_mode {
            Some(PolledModeQueue::new(100)) // Queue up to 100 frames
        } else {
            None
        };
        
        let serial_clone = Arc::clone(&serial_port);
        let mut read_stream = stream.try_clone()
            .expect("Failed to clone stream");
        let logger_clone = Arc::clone(logger);
        let cc_clone = cc_config.clone();
        let pcap_clone = pcap_writer.clone();
        let port_cfg_clone = port_config.clone();
        let polled_queue_clone = polled_queue.as_ref().map(|q| q.clone_arc());
        
        // Spawn Serial → TCP thread
        let serial_to_tcp = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            let mut frame_buffer = KissFrameBuffer::new();
            
            loop {
                let mut port = serial_clone.lock().unwrap();
                match port.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        drop(port);
                        
                        let frames = frame_buffer.add_bytes(&buffer[..n]);
                        
                        for mut frame in frames {
                            // Verify checksum if enabled
                            if port_cfg_clone.extended_kiss && port_cfg_clone.checksum_mode {
                                frame = match verify_and_remove_checksum(&frame) {
                                    Some(f) => f,
                                    None => {
                                        logger_clone.log("Checksum verification failed", 4);
                                        continue;
                                    }
                                };
                            }
                            
                            if let Some((port_num, _, _)) = extract_kiss_info(&frame) {
                                if port_num == kiss_port {
                                    let processed = if cc_clone.phil_flag {
                                        process_frame_with_phil_flag(&frame)
                                    } else { 
                                        frame 
                                    };
                                    
                                    if cc_clone.parse_kiss {
                                        parse_kiss_frame_static(
                                            &processed, 
                                            "Serial->TCP", 
                                            &pcap_clone,
                                            cc_clone.dump_ax25
                                        );
                                    } else if cc_clone.dump_frames {
                                        dump_frame(&processed, "Serial->TCP");
                                    }
                                    
                                    // If polled mode, queue the frame instead of sending
                                    if let Some(ref queue) = polled_queue_clone {
                                        let mut q = queue.lock().unwrap();
                                        if q.len() < 100 {
                                            q.push_back(processed);
                                        }
                                    } else {
                                        // Standard mode: send immediately
                                        if let Err(e) = read_stream.write_all(&processed) {
                                            logger_clone.log(
                                                &format!("Error writing to TCP: {}", e), 
                                                3
                                            );
                                            return;
                                        }
                                    }
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
                        logger_clone.log(
                            &format!("Serial read error: {}", e), 
                            3
                        ); 
                        break; 
                    }
                }
            }
        });
        
        // Start polling thread if in polled mode
        if let Some(ref queue) = polled_queue {
            let queue_clone = queue.clone_arc();
            let mut stream_clone = stream.try_clone().expect("Failed to clone stream");
            let poll_interval = port_config.poll_interval_ms;
            let logger_poll = Arc::clone(logger);
            let port_cfg = port_config.clone();
            
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_millis(poll_interval));
                    
                    // Send poll frame
                    let poll_frame = create_poll_frame(kiss_port);
                    let poll_to_send = if port_cfg.checksum_mode {
                        add_kiss_checksum(&poll_frame)
                    } else {
                        poll_frame
                    };
                    
                    // Check if there's data to send
                    let frame_to_send = {
                        let mut q = queue_clone.lock().unwrap();
                        q.pop_front()
                    };
                    
                    if let Some(frame) = frame_to_send {
                        // Send queued frame
                        if let Err(e) = stream_clone.write_all(&frame) {
                            logger_poll.log(&format!("Poll send error: {}", e), 3);
                            break;
                        }
                    } else {
                        // No data, send empty poll response
                        if let Err(e) = stream_clone.write_all(&poll_to_send) {
                            logger_poll.log(&format!("Poll response error: {}", e), 3);
                            break;
                        }
                    }
                }
            });
        }
        
        // Main thread handles TCP → Serial
        let mut buffer = [0u8; 1024];
        let mut frame_buffer = KissFrameBuffer::new();
        
        loop {
            match stream.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let frames = frame_buffer.add_bytes(&buffer[..n]);
                    
                    for mut frame in frames {
                        // Verify checksum if enabled
                        if port_config.extended_kiss && port_config.checksum_mode {
                            frame = match verify_and_remove_checksum(&frame) {
                                Some(f) => f,
                                None => {
                                    logger.log("Checksum verification failed (TCP->Serial)", 4);
                                    continue;
                                }
                            };
                        }
                        
                        // Check for acknowledgment required frame
                        if port_config.extended_kiss && is_ack_required_frame(&frame) {
                            // Create and send acknowledgment
                            if let Some(ack) = create_ack_frame(&frame) {
                                let ack_to_send = if port_config.checksum_mode {
                                    add_kiss_checksum(&ack)
                                } else {
                                    ack
                                };
                                
                                // Send ACK back to TCP client
                                let _ = stream.write_all(&ack_to_send);
                            }
                        }
                        
                        // Modify KISS port number
                        let modified = modify_kiss_port(&frame, kiss_port);
                        
                        // Apply PhilFlag if configured
                        let mut processed = if cc_config.phil_flag {
                            process_phil_flag_tcp_to_serial(&modified)
                        } else { 
                            modified 
                        };
                        
                        // Add checksum if enabled
                        if port_config.extended_kiss && port_config.checksum_mode {
                            processed = add_kiss_checksum(&processed);
                        }
                        
                        if cc_config.parse_kiss {
                            parse_kiss_frame_static(
                                &processed, 
                                "TCP->Serial", 
                                pcap_writer,
                                cc_config.dump_ax25
                            );
                        }
                        
                        // Send to serial port
                        let mut port = serial_port.lock().unwrap();
                        if let Err(e) = port.write_all(&processed) {
                            logger.log(
                                &format!("Serial write error: {}", e), 
                                3
                            );
                            break;
                        }
                    }
                }
                Ok(_) => { 
                    logger.log("Client disconnected", 5); 
                    break; 
                }
                Err(e) => { 
                    logger.log(
                        &format!("TCP read error: {}", e), 
                        3
                    ); 
                    break; 
                }
            }
        }
        
        drop(serial_to_tcp);
    }
    
    fn start_serial_to_serial(
        &self, 
        cc: &CrossConnect, 
        id_a: &str, 
        port_a: u8,
        id_b: &str, 
        port_b: u8
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        let serial_a = { 
            self.serial_manager.lock().unwrap().get_port(id_a) 
        }.ok_or(format!("Serial port {} not found", id_a))?;
        
        let serial_b = { 
            self.serial_manager.lock().unwrap().get_port(id_b) 
        }.ok_or(format!("Serial port {} not found", id_b))?;
        
        let translator_a_to_b = KissPortTranslator::new(port_a, port_b);
        let translator_b_to_a = KissPortTranslator::new(port_b, port_a);
        
        let _logger = Arc::clone(&self.logger);
        let pcap_a = self.pcap_writer.clone();
        let pcap_b = self.pcap_writer.clone();
        let cc_a = cc.clone();
        let cc_b = cc.clone();
        
        let a = Arc::clone(&serial_a);
        let b = Arc::clone(&serial_b);
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            let mut fb = KissFrameBuffer::new();
            
            loop {
                let mut port = a.lock().unwrap();
                match port.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        drop(port);
                        
                        for frame in fb.add_bytes(&buf[..n]) {
                            if let Some(trans) = translator_a_to_b.translate(&frame) {
                                if cc_a.parse_kiss {
                                    parse_kiss_frame_static(&trans, "Serial A->B", &pcap_a, cc_a.dump_ax25);
                                }
                                
                                let mut p = b.lock().unwrap();
                                let _ = p.write_all(&trans);
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
                    Err(_) => break,
                }
            }
        });
        
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            let mut fb = KissFrameBuffer::new();
            
            loop {
                let mut port = serial_b.lock().unwrap();
                match port.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        drop(port);
                        
                        for frame in fb.add_bytes(&buf[..n]) {
                            if let Some(trans) = translator_b_to_a.translate(&frame) {
                                if cc_b.parse_kiss {
                                    parse_kiss_frame_static(&trans, "Serial B->A", &pcap_b, cc_b.dump_ax25);
                                }
                                
                                let mut p = serial_a.lock().unwrap();
                                let _ = p.write_all(&trans);
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
                    Err(_) => break,
                }
            }
        });
        
        Ok(())
    }
    
    fn handle_raw_copy(
        mut stream: TcpStream, 
        serial: Arc<Mutex<Box<dyn serialport::SerialPort>>>, 
        logger: &Arc<Logger>
    ) {
        let s = Arc::clone(&serial);
        let mut rs = stream.try_clone().unwrap();
        let l = Arc::clone(logger);
        
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                match s.lock().unwrap().read(&mut buf) {
                    Ok(n) if n > 0 => { 
                        if rs.write_all(&buf[..n]).is_err() { 
                            break; 
                        } 
                    }
                    Ok(_) => thread::sleep(Duration::from_millis(10)),
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        thread::sleep(Duration::from_millis(10))
                    }
                    Err(e) => { 
                        l.log(&format!("Raw serial read: {}", e), 3); 
                        break; 
                    }
                }
            }
        });
        
        let mut buf = [0u8; 1024];
        loop {
            match stream.read(&mut buf) {
                Ok(n) if n > 0 => { 
                    if serial.lock().unwrap().write_all(&buf[..n]).is_err() { 
                        break; 
                    } 
                }
                Ok(_) => { 
                    logger.log("Raw client disconnected", 5); 
                    break; 
                }
                Err(e) => { 
                    logger.log(&format!("Raw TCP read: {}", e), 3); 
                    break; 
                }
            }
        }
    }
}
// ==============================================================================
// MAIN.RS - PART 5 OF 5
// ==============================================================================
//
// This part contains:
// - Main function
// - Command-line argument parsing
// - Configuration loading
// - Startup sequence
//
// ==============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("\nReceived SIGINT, shutting down gracefully...");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
        process::exit(0);
    })?;
    
    let args: Vec<String> = std::env::args().collect();
    
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("rax25kb v1.6.3 - Multi-Port Cross-Connect KISS Bridge");
        println!();
        println!("Usage: {} [OPTIONS]", args[0]);
        println!();
        println!("Options:");
        println!("  -c <file>  Configuration file (default: rax25kb.cfg)");
        println!("  -q         Quiet startup (suppress banner)");
        println!("  -h         Show this help");
        println!();
        println!("Configuration:");
        println!("  See rax25kb.cfg(5) for configuration file syntax");
        println!();
        println!("Examples:");
        println!("  {} -c /etc/rax25kb.cfg", args[0]);
        println!("  {} -q", args[0]);
        println!();
        println!("Documentation:");
        println!("  man rax25kb           # Program documentation");
        println!("  man rax25kb.cfg       # Configuration file syntax");
        println!();
        println!("For more information:");
        println!("  https://github.com/ke4ahr/rax25kb");
        println!("  https://www.outpostpm.org/support.html");
        println!();
        return Ok(());
    }
    
    let config_file = args.iter()
        .position(|a| a == "-c")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("rax25kb.cfg");
    
    let quiet = args.iter().any(|a| a == "-q" || a == "--quiet");
    
    let config = match Config::from_file(config_file) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration from '{}': {}", config_file, e);
            eprintln!();
            eprintln!("Please check:");
            eprintln!("  - File exists and is readable");
            eprintln!("  - Configuration syntax is correct");
            eprintln!("  - Serial ports are defined before cross-connects");
            eprintln!("  - KISS port numbers are 0-15");
            eprintln!("  - Serial port IDs match in cross-connect definitions");
            eprintln!();
            eprintln!("See 'man rax25kb.cfg' for configuration help");
            eprintln!();
            eprintln!("Example minimal configuration:");
            eprintln!("  serial_port0000=/dev/ttyUSB0");
            eprintln!("  serial_port0000_baud=9600");
            eprintln!("  cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001");
            eprintln!();
            process::exit(1);
        }
    };
    
    if config.serial_ports.is_empty() {
        eprintln!("Error: No serial ports configured");
        eprintln!();
        eprintln!("Add at least one serial port to your configuration:");
        eprintln!("  serial_port0000=/dev/ttyUSB0");
        eprintln!("  serial_port0000_baud=9600");
        eprintln!();
        eprintln!("Linux devices:   /dev/ttyUSB0, /dev/ttyACM0");
        eprintln!("Windows devices: COM3, COM4");
        eprintln!("macOS devices:   /dev/cu.usbserial-*");
        eprintln!();
        process::exit(1);
    }
    
    if config.cross_connects.is_empty() {
        eprintln!("Error: No cross-connects configured");
        eprintln!();
        eprintln!("Add at least one cross-connect to your configuration:");
        eprintln!("  cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001");
        eprintln!();
        eprintln!("Format: endpoint_a <-> endpoint_b");
        eprintln!("  TCP endpoint:    tcp:address:port");
        eprintln!("  Serial endpoint: serial:port_id:kiss_port");
        eprintln!();
        eprintln!("Note: Most TNCs use KISS port 0 (the default)");
        eprintln!();
        process::exit(1);
    }
    
    if !quiet && !config.quiet_startup {
        println!("rax25kb v1.6.3 - Multi-Port Cross-Connect KISS Bridge");
        println!("======================================================");
        println!();
        
        println!("Serial ports configured: {}", config.serial_ports.len());
        for (id, port) in &config.serial_ports {
            println!("  [{}] {} @ {} baud", id, port.device, port.baud_rate);
            
            if port.flow_control != FlowControl::None {
                println!("       Flow control: {:?}", port.flow_control);
            }
            if port.stop_bits != StopBits::One {
                println!("       Stop bits: {:?}", port.stop_bits);
            }
            if port.parity != Parity::None {
                println!("       Parity: {:?}", port.parity);
            }
            if port.extended_kiss {
                println!("       Extended KISS enabled");
            }
        }
        println!();
        
        println!("Cross-connects configured: {}", config.cross_connects.len());
        for cc in &config.cross_connects {
            println!("  [{}] {:?} <-> {:?}", 
                cc.id, cc.endpoint_a, cc.endpoint_b);
            
            if cc.phil_flag {
                println!("       PhilFlag correction enabled");
            }
            if cc.parse_kiss {
                println!("       KISS frame parsing enabled");
            }
            if cc.dump_frames {
                println!("       Frame dumping enabled");
            }
            if cc.raw_copy {
                println!("       Raw copy mode (no KISS processing)");
            }
        }
        println!();
    }
    
    if let Some(ref pidfile) = config.pidfile {
        match File::create(pidfile) {
            Ok(mut f) => {
                if let Err(e) = writeln!(f, "{}", process::id()) {
                    eprintln!("Warning: Failed to write PID to '{}': {}", pidfile, e);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to create PID file '{}': {}", pidfile, e);
            }
        }
    }
    
    let logger = Arc::new(Logger::new(
        config.logfile.clone(),
        config.log_level,
        config.log_to_console,
    )?);
    
    logger.log("rax25kb v1.6.3 starting", 5);
    logger.log(&format!("Configuration loaded from: {}", config_file), 6);
    logger.log(&format!("Serial ports: {}", config.serial_ports.len()), 6);
    logger.log(&format!("Cross-connects: {}", config.cross_connects.len()), 6);
    logger.log(&format!("Log level: {}", config.log_level), 6);
    
    let pcap_writer = if let Some(ref pcap_path) = config.pcap_file {
        match PcapWriter::new(pcap_path) {
            Ok(writer) => {
                logger.log(&format!("PCAP capture enabled: {}", pcap_path), 5);
                Some(Arc::new(writer))
            }
            Err(e) => {
                logger.log(
                    &format!("Warning: Failed to create PCAP file '{}': {}", pcap_path, e), 
                    4
                );
                eprintln!("Warning: PCAP capture disabled due to error: {}", e);
                None
            }
        }
    } else {
        None
    };
    
    logger.log("Initializing cross-connect manager", 5);
    
    let manager = match CrossConnectManager::new(
        config, 
        logger.clone(), 
        pcap_writer
    ) {
        Ok(mgr) => mgr,
        Err(e) => {
            eprintln!("Error initializing cross-connect manager: {}", e);
            eprintln!();
            eprintln!("Common causes:");
            eprintln!("  - Serial port doesn't exist or wrong path");
            eprintln!("  - Serial port already in use by another program");
            eprintln!("  - Insufficient permissions (try: sudo usermod -a -G dialout $USER)");
            eprintln!("  - Invalid baud rate or serial settings");
            eprintln!();
            logger.log(&format!("Fatal error during initialization: {}", e), 0);
            process::exit(1);
        }
    };
    
    logger.log("Starting all cross-connects", 5);
    
    if let Err(e) = manager.start_all() {
        eprintln!("Error starting cross-connects: {}", e);
        eprintln!();
        eprintln!("Common causes:");
        eprintln!("  - TCP port already in use (check with: netstat -tln)");
        eprintln!("  - Invalid TCP address or port number");
        eprintln!("  - Firewall blocking the port");
        eprintln!();
        logger.log(&format!("Fatal error starting cross-connects: {}", e), 0);
        process::exit(1);
    }
    
    logger.log("All cross-connects started successfully", 5);
    logger.log("Entering main loop", 6);
    
    if !quiet {
        println!("rax25kb is running. Press Ctrl+C to stop.");
        println!();
        println!("Monitoring:");
        for cc in &manager.config.cross_connects {
            match (&cc.endpoint_a, &cc.endpoint_b) {
                (_, CrossConnectEndpoint::TcpSocket { address, port }) |
                (CrossConnectEndpoint::TcpSocket { address, port }, _) => {
                    println!("  Cross-connect {}: tcp://{}:{}", cc.id, address, port);
                }
                _ => {}
            }
        }
        println!();
        
        if let Some(ref logfile) = manager.config.logfile {
            println!("Log file: {}", logfile);
        }
        if let Some(ref pcap) = manager.config.pcap_file {
            println!("PCAP file: {}", pcap);
        }
        println!();
    }
    
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
