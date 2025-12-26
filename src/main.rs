// rax25kb - AX.25 KISS Bridge with Multi-Port Cross-Connect Support
// Version 2.0.0
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

use std::collections::HashMap;      // For storing serial port and config maps
use std::fs::{self, File, OpenOptions}; // File system operations
use std::io::{Read, Write};         // I/O traits for reading/writing
use std::net::{TcpListener, TcpStream}; // TCP networking
use std::sync::{Arc, Mutex};        // Thread-safe reference counting and locking
use std::thread;                    // Threading support
use std::time::Duration;            // Time duration for timeouts
use std::process;                   // Process control (exit, pid)

// ==============================================================================
// KISS PROTOCOL CONSTANTS
// ==============================================================================
// KISS (Keep It Simple Stupid) is a protocol for communicating with TNCs
// (Terminal Node Controllers) over serial connections.
//
// Frame format: FEND [port+cmd] [data] FEND
// Where:
//   FEND = Frame End marker (0xC0)
//   port+cmd = 4-bit port number (upper nibble) + 4-bit command (lower nibble)
//   data = Actual packet data (may contain escaped bytes)
//
// Byte stuffing:
//   FEND in data -> FESC TFEND (0xDB 0xDC)
//   FESC in data -> FESC TFESC (0xDB 0xDD)

const KISS_FEND: u8 = 0xC0;         // Frame End - marks start/end of frame
const KISS_FESC: u8 = 0xDB;         // Frame Escape - escape sequence marker
const KISS_TFEND: u8 = 0xDC;        // Transposed Frame End - escaped FEND
#[allow(dead_code)]
const KISS_TFESC: u8 = 0xDD;        // Transposed Frame Escape - escaped FESC

// ==============================================================================
// CONFIGURATION STRUCTURES
// ==============================================================================

/// Flow control types for serial ports
#[derive(Debug, Clone, Copy, PartialEq)]
enum FlowControl {
    None,      // No flow control (most common)
    Software,  // XON/XOFF software flow control
    Hardware,  // RTS/CTS hardware flow control
    DtrDsr,    // DTR/DSR flow control (primarily Windows)
}

/// Number of stop bits for serial communication
#[derive(Debug, Clone, Copy, PartialEq)]
enum StopBits {
    One,       // 1 stop bit (most common)
    Two,       // 2 stop bits (e.g., Kenwood TS-2000)
}

/// Parity checking for serial communication
#[derive(Debug, Clone, Copy, PartialEq)]
enum Parity {
    None,      // No parity (most common - 8N1)
    Odd,       // Odd parity
    Even,      // Even parity
}

/// Represents an endpoint in a cross-connect
/// Can be either a TCP socket or a serial port with KISS port number
#[derive(Debug, Clone, PartialEq)]
enum CrossConnectEndpoint {
    /// TCP socket endpoint (server mode - listens for connections)
    TcpSocket {
        address: String,  // Bind address (0.0.0.0 = all, 127.0.0.1 = localhost)
        port: u16,        // TCP port number
    },
    /// Serial port endpoint with KISS port addressing
    SerialPort {
        port_id: String,  // ID of the serial port (references serial_portXXXX)
        kiss_port: u8,    // KISS port number (0-15, most TNCs use 0)
    },
}

/// Configuration for a single serial port
#[derive(Debug, Clone)]
struct SerialPortConfig {
    id: String,              // Unique 4-digit ID (e.g., "0000", "0001")
    device: String,          // Device path (/dev/ttyUSB0, COM3, etc.)
    baud_rate: u32,          // Baud rate (1200, 9600, 115200, etc.)
    flow_control: FlowControl, // Flow control type
    stop_bits: StopBits,     // Number of stop bits
    parity: Parity,          // Parity setting
    extended_kiss: bool,     // true = Extended KISS, false = Standard KISS
}

/// Configuration for a cross-connect between two endpoints
#[derive(Debug, Clone)]
struct CrossConnect {
    id: String,                     // Unique 4-digit ID (e.g., "0000", "0001")
    endpoint_a: CrossConnectEndpoint, // First endpoint
    endpoint_b: CrossConnectEndpoint, // Second endpoint
    phil_flag: bool,                // Enable PhilFlag correction for buggy TNCs
    dump_frames: bool,              // Dump frames in hex format
    parse_kiss: bool,               // Parse and display KISS frames
    dump_ax25: bool,                // Display AX.25 frame details
    raw_copy: bool,                 // Raw byte copy mode (no KISS processing)
}

/// Main configuration structure loaded from config file
#[derive(Debug, Clone)]
struct Config {
    serial_ports: HashMap<String, SerialPortConfig>, // All serial port configs
    cross_connects: Vec<CrossConnect>,               // All cross-connect configs
    log_level: u8,                                   // Logging verbosity (0-9)
    logfile: Option<String>,                         // Log file path (optional)
    pidfile: Option<String>,                         // PID file path (optional)
    log_to_console: bool,                            // Log to stdout
    quiet_startup: bool,                             // Suppress startup banner
    pcap_file: Option<String>,                       // PCAP capture file (optional)
}

// ==============================================================================
// AX.25 PROTOCOL STRUCTURES
// ==============================================================================
// AX.25 is the data link layer protocol used in amateur packet radio.
// It's based on X.25 but adapted for amateur radio use.
//
// Frame structure:
//   [Destination Address (7 bytes)]
//   [Source Address (7 bytes)]
//   [Digipeater Addresses (0-8, 7 bytes each)]
//   [Control (1 byte)]
//   [PID (0-1 bytes)]
//   [Information Field (variable)]
//
// Address format (7 bytes):
//   Bytes 0-5: Callsign (left-shifted by 1)
//   Byte 6: SSID + reserved bits + extension bit

/// AX.25 address structure (callsign + SSID)
#[derive(Debug)]
struct AX25Address {
    callsign: String,  // Station callsign (e.g., "N0CALL")
    ssid: u8,          // Secondary Station ID (0-15)
}

impl AX25Address {
    /// Parse an AX.25 address from 7 bytes
    /// 
    /// AX.25 addresses are encoded with each character shifted left by 1 bit.
    /// The 7th byte contains the SSID in bits 1-4.
    fn from_ax25_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 7 { 
            return None; 
        }
        
        // Extract callsign from bytes 0-5 (right-shift to get ASCII)
        let mut callsign = String::new();
        for i in 0..6 {
            let ch = (bytes[i] >> 1) as char;
            if ch != ' ' {  // Skip padding spaces
                callsign.push(ch); 
            }
        }
        
        // Extract SSID from byte 6 (bits 1-4)
        let ssid = (bytes[6] >> 1) & 0x0F;
        
        Some(AX25Address { callsign, ssid })
    }
    
    /// Convert address to string format (e.g., "N0CALL" or "N0CALL-5")
    fn to_string(&self) -> String {
        if self.ssid == 0 { 
            self.callsign.clone() 
        } else { 
            format!("{}-{}", self.callsign, self.ssid) 
        }
    }
}

/// Complete AX.25 frame structure
#[derive(Debug)]
struct AX25Frame {
    destination: AX25Address,      // Destination station
    source: AX25Address,           // Source station
    digipeaters: Vec<AX25Address>, // Digipeater path (0-8 stations)
    control: u8,                   // Control byte (frame type, sequence numbers)
    pid: Option<u8>,               // Protocol ID (present in I and UI frames)
    info: Vec<u8>,                 // Information field (payload data)
}

/// AX.25 frame types based on control byte
#[derive(Debug, PartialEq)]
enum AX25FrameType {
    IFrame,   // Information frame (bit 0 = 0)
    SFrame,   // Supervisory frame (bits 0-1 = 01)
    UFrame,   // Unnumbered frame (bits 0-1 = 11, not UI)
    UIFrame,  // Unnumbered Information frame (special case)
    Unknown,  // Unrecognized frame type
}

impl AX25Frame {
    /// Parse an AX.25 frame from raw bytes
    /// 
    /// Returns None if the frame is invalid or too short
    fn parse(data: &[u8]) -> Option<Self> {
        // Minimum frame: dest(7) + source(7) + control(1) + FCS(2) = 17 bytes
        // We check for 16 here since FCS might be stripped
        if data.len() < 16 { 
            return None; 
        }
        
        let mut offset = 0;
        
        // Parse destination address (7 bytes)
        let destination = AX25Address::from_ax25_bytes(&data[offset..offset+7])?;
        offset += 7;
        
        // Parse source address (7 bytes)
        let source = AX25Address::from_ax25_bytes(&data[offset..offset+7])?;
        offset += 7;
        
        // Parse digipeater addresses (optional, 0-8 stations)
        // Continue until we find an address with the extension bit set (bit 0 of byte 6)
        let mut digipeaters = Vec::new();
        while offset + 7 <= data.len() {
            let addr_byte_6 = data[offset + 6];
            let digi = AX25Address::from_ax25_bytes(&data[offset..offset+7])?;
            digipeaters.push(digi);
            offset += 7;
            
            // Extension bit set means this is the last address
            if addr_byte_6 & 0x01 != 0 { 
                break; 
            }
        }
        
        // Ensure we have room for control byte
        if offset >= data.len() { 
            return None; 
        }
        
        // Parse control byte
        let control = data[offset];
        offset += 1;
        
        // Parse PID (Protocol ID) if present
        // PID is present in I-frames (control & 0x01 == 0) and UI-frames (control == 0x03)
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
        
        // Remaining bytes are the information field
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
    
    /// Determine the frame type from the control byte
    /// 
    /// Control byte format:
    ///   I-frame:  N(R) P N(S) 0
    ///   S-frame:  N(R) P/F S S 0 1
    ///   U-frame:  M M P/F M M 1 1
    fn get_frame_type(&self) -> AX25FrameType {
        if (self.control & 0x01) == 0 {
            // Bit 0 = 0: I-frame
            AX25FrameType::IFrame 
        } else if (self.control & 0x03) == 0x01 {
            // Bits 0-1 = 01: S-frame
            AX25FrameType::SFrame 
        } else if (self.control & 0x03) == 0x03 {
            // Bits 0-1 = 11: U-frame or UI-frame
            if (self.control & 0xEF) == 0x03 { 
                // UI frame (special unnumbered information)
                AX25FrameType::UIFrame 
            } else { 
                // Other unnumbered frame
                AX25FrameType::UFrame 
            }
        } else { 
            AX25FrameType::Unknown 
        }
    }
    
    /// Get a human-readable description of the connection phase
    fn get_connection_phase(&self) -> &str {
        match self.get_frame_type() {
            AX25FrameType::IFrame => "CONNECTED (Information Transfer)",
            AX25FrameType::SFrame => "CONNECTED (Supervisory)",
            AX25FrameType::UFrame => {
                // Decode specific U-frame types
                match self.control & 0xEF {
                    0x2F => "SETUP (SABM)",      // Set Asynchronous Balanced Mode
                    0x63 => "SETUP (SABME)",     // SABM Extended
                    0x43 => "DISCONNECT (DISC)", // Disconnect
                    0x0F => "DISCONNECT (DM)",   // Disconnected Mode
                    0x87 => "ERROR (FRMR)",      // Frame Reject
                    _ => "CONTROL (Unnumbered)",
                }
            }
            AX25FrameType::UIFrame => "UNCONNECTED (UI Frame)",
            AX25FrameType::Unknown => "UNKNOWN",
        }
    }
    
    /// Print a human-readable summary of the frame
    fn print_summary(&self) {
        // Print source -> destination
        println!("  AX.25: {} > {}", 
            self.source.to_string(), 
            self.destination.to_string()
        );
        
        // Print digipeater path if present
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
        
        // Print frame type and phase
        println!("  Type: {:?}", self.get_frame_type());
        println!("  Phase: {}", self.get_connection_phase());
        
        // Print control byte
        println!("  Control: 0x{:02x}", self.control);
        
        // Print PID if present
        if let Some(pid) = self.pid { 
            println!("  PID: 0x{:02x}", pid); 
        }
        
        // Print info field length
        if !self.info.is_empty() { 
            println!("  Info: {} bytes", self.info.len()); 
        }
    }
}

// ==============================================================================
// END OF PART 1
// ==============================================================================
// Continue with Part 2: Configuration parsing// ==============================================================================
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
// Configuration file format:
//   key=value format with # comments
//   serial_portXXXX entries define serial ports
//   cross_connectXXXX entries define connections
//   global settings control logging and behavior
//
// ==============================================================================

impl Config {
    /// Load configuration from a file
    /// 
    /// File format is simple key=value pairs with optional quotes:
    ///   key=value
    ///   key="value with spaces"
    ///   # comments
    /// 
    /// Returns an error if the file cannot be read or parsing fails
    fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Read the entire configuration file
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file '{}': {}", path, e))?;
        
        // Parse into a HashMap for easy lookup
        let mut config_map = HashMap::new();
        
        // Process each line
        for line in contents.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Split on first '=' character
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let mut value = value.trim();
                
                // Strip quotes if present
                if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                    value = &value[1..value.len()-1];
                }
                
                config_map.insert(key.to_string(), value.to_string());
            }
        }
        
        // ==================================================================
        // PARSE SERIAL PORT CONFIGURATIONS
        // ==================================================================
        
        let mut serial_ports = HashMap::new();
        let mut serial_port_ids = Vec::new();
        
        // Find all serial_portXXXX entries
        // Look for keys like "serial_port0000", "serial_port0001", etc.
        for key in config_map.keys() {
            if key.starts_with("serial_port") && key.len() > 11 {
                let id = &key[11..];
                
                // Validate ID is exactly 4 digits
                if id.chars().all(|c| c.is_ascii_digit()) && id.len() == 4 {
                    if !serial_port_ids.contains(&id.to_string()) {
                        serial_port_ids.push(id.to_string());
                    }
                }
            }
        }
        
        // Parse configuration for each serial port
        for id in &serial_port_ids {
            // Required: device path
            let device_key = format!("serial_port{}", id);
            let device = config_map.get(&device_key)
                .ok_or(format!("Missing device for serial port {}", id))?
                .clone();
            
            // Optional: baud rate (default 9600)
            let baud_key = format!("serial_port{}_baud", id);
            let baud_rate = config_map.get(&baud_key)
                .and_then(|v| v.parse().ok())
                .unwrap_or(9600);
            
            // Optional: flow control (default none)
            let flow_key = format!("serial_port{}_flow_control", id);
            let flow_control = config_map.get(&flow_key)
                .and_then(|v| Self::parse_flow_control(v))
                .unwrap_or(FlowControl::None);
            
            // Optional: stop bits (default 1)
            let stop_key = format!("serial_port{}_stop_bits", id);
            let stop_bits = config_map.get(&stop_key)
                .and_then(|v| Self::parse_stop_bits(v))
                .unwrap_or(StopBits::One);
            
            // Optional: parity (default none)
            let parity_key = format!("serial_port{}_parity", id);
            let parity = config_map.get(&parity_key)
                .and_then(|v| Self::parse_parity(v))
                .unwrap_or(Parity::None);
            
            // Optional: extended KISS (default false)
            let xkiss_key = format!("serial_port{}_extended_kiss", id);
            let extended_kiss = config_map.get(&xkiss_key)
                .and_then(|v| Self::parse_bool(v))
                .unwrap_or(false);
            
            // Create SerialPortConfig and add to map
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
        
        // ==================================================================
        // PARSE CROSS-CONNECT CONFIGURATIONS
        // ==================================================================
        
        let mut cross_connects = Vec::new();
        let mut cross_connect_ids = Vec::new();
        
        // Find all cross_connectXXXX entries
        // Look for keys like "cross_connect0000", "cross_connect0001", etc.
        for key in config_map.keys() {
            if key.starts_with("cross_connect") && key.len() == 17 {
                let id = &key[13..17];
                
                // Validate ID is exactly 4 digits
                if id.chars().all(|c| c.is_ascii_digit()) {
                    if !cross_connect_ids.contains(&id.to_string()) {
                        cross_connect_ids.push(id.to_string());
                    }
                }
            }
        }
        
        // Sort IDs to process in numerical order
        cross_connect_ids.sort();
        
        // Parse configuration for each cross-connect
        for id in cross_connect_ids {
            // Required: cross-connect definition
            let cc_key = format!("cross_connect{}", id);
            let cc_value = config_map.get(&cc_key)
                .ok_or(format!("Missing cross_connect{}", id))?;
            
            // Parse cross-connect value: "endpoint_a <-> endpoint_b"
            let parts: Vec<&str> = cc_value.split("<->").collect();
            if parts.len() != 2 {
                return Err(format!(
                    "Invalid cross_connect{} format: {} (expected: endpoint <-> endpoint)", 
                    id, cc_value
                ).into());
            }
            
            // Parse both endpoints
            let endpoint_a = Self::parse_endpoint(parts[0].trim(), &serial_ports)?;
            let endpoint_b = Self::parse_endpoint(parts[1].trim(), &serial_ports)?;
            
            // Parse optional feature flags
            
            // PhilFlag: fixes KISS bugs in TASCO modems and TS-2000
            let phil_key = format!("cross_connect{}_phil_flag", id);
            let phil_flag = config_map.get(&phil_key)
                .and_then(|v| Self::parse_bool(v))
                .unwrap_or(false);
            
            // Dump frames: show hex dump of all frames
            let dump_key = format!("cross_connect{}_dump", id);
            let dump_frames = config_map.get(&dump_key)
                .and_then(|v| Self::parse_bool(v))
                .unwrap_or(false);
            
            // Parse KISS: decode and display KISS frame information
            let parse_key = format!("cross_connect{}_parse_kiss", id);
            let parse_kiss = config_map.get(&parse_key)
                .and_then(|v| Self::parse_bool(v))
                .unwrap_or(false);
            
            // Dump AX.25: show decoded AX.25 frame details
            let ax25_key = format!("cross_connect{}_dump_ax25", id);
            let dump_ax25 = config_map.get(&ax25_key)
                .and_then(|v| Self::parse_bool(v))
                .unwrap_or(false);
            
            // Raw copy: bypass KISS processing, copy bytes directly
            let raw_key = format!("cross_connect{}_raw_copy", id);
            let raw_copy = config_map.get(&raw_key)
                .and_then(|v| Self::parse_bool(v))
                .unwrap_or(false);
            
            // Create CrossConnect and add to vector
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
        
        // ==================================================================
        // CREATE DEFAULT CROSS-CONNECT IF NONE DEFINED
        // ==================================================================
        
        // If no cross-connects are defined but we have serial ports,
        // create a default cross-connect: serial:0000:0 <-> tcp:0.0.0.0:8001
        if cross_connects.is_empty() && !serial_ports.is_empty() {
            let first_port_id = serial_ports.keys().next().unwrap().clone();
            
            let default_cc = CrossConnect {
                id: "0000".to_string(),
                endpoint_a: CrossConnectEndpoint::SerialPort {
                    port_id: first_port_id,
                    kiss_port: 0,  // Default KISS port
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
        
        // ==================================================================
        // PARSE GLOBAL SETTINGS
        // ==================================================================
        
        // Log level (0-9, default 5 = NOTICE)
        let log_level = config_map.get("log_level")
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        
        // Optional file paths
        let logfile = config_map.get("logfile").cloned();
        let pidfile = config_map.get("pidfile").cloned();
        let pcap_file = config_map.get("pcap_file").cloned();
        
        // Boolean flags
        let log_to_console = config_map.get("log_to_console")
            .and_then(|v| Self::parse_bool(v))
            .unwrap_or(true);
        
        let quiet_startup = config_map.get("quiet_startup")
            .and_then(|v| Self::parse_bool(v))
            .unwrap_or(false);
        
        // Return complete configuration
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
    
    /// Parse an endpoint specification string
    /// 
    /// Format: "tcp:address:port" or "serial:port_id:kiss_port"
    /// 
    /// Examples:
    ///   "tcp:0.0.0.0:8001"         - TCP on all interfaces, port 8001
    ///   "tcp:127.0.0.1:8002"       - TCP on localhost only, port 8002
    ///   "serial:0000:0"            - Serial port 0000, KISS port 0
    ///   "serial:0001:3"            - Serial port 0001, KISS port 3
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
                // TCP endpoint: tcp:address:port
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
                // Serial endpoint: serial:port_id:kiss_port
                if parts.len() != 3 {
                    return Err(format!(
                        "Invalid serial endpoint format: {} (expected: serial:port_id:kiss_port)", 
                        s
                    ).into());
                }
                
                let port_id = parts[1].to_string();
                let kiss_port = parts[2].parse::<u8>()
                    .map_err(|_| format!("Invalid KISS port: {}", parts[2]))?;
                
                // Validate KISS port range (0-15)
                if kiss_port > 15 {
                    return Err(format!(
                        "KISS port must be 0-15, got: {}", 
                        kiss_port
                    ).into());
                }
                
                // Validate that serial port exists
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
    
    /// Parse flow control string
    /// 
    /// Accepts: none, software, hardware, dtrdsr (and common aliases)
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
    
    /// Parse stop bits string
    /// 
    /// Accepts: 1, 2, one, two
    fn parse_stop_bits(s: &str) -> Option<StopBits> {
        match s {
            "1" | "one" => Some(StopBits::One),
            "2" | "two" => Some(StopBits::Two),
            _ => None,
        }
    }
    
    /// Parse parity string
    /// 
    /// Accepts: none, even, odd (and common aliases)
    fn parse_parity(s: &str) -> Option<Parity> {
        match s.to_lowercase().as_str() {
            "none" | "n" | "no" => Some(Parity::None),
            "odd" | "o" => Some(Parity::Odd),
            "even" | "e" => Some(Parity::Even),
            _ => None,
        }
    }
    
    /// Parse boolean string
    /// 
    /// Accepts many formats: true/false, yes/no, on/off, 1/0
    fn parse_bool(s: &str) -> Option<bool> {
        match s.to_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        }
    }
}

// ==============================================================================
// END OF PART 2
// ==============================================================================
// Continue with Part 3: Logger, PCAP writer, KISS handling// ==============================================================================
// MAIN.RS - PART 3 OF 5
// ==============================================================================
//
// This part contains:
// - Logger implementation (file and console logging)
// - PCAP writer (Wireshark-compatible packet capture)
// - KISS frame buffer (frame boundary detection)
// - KISS helper functions (port extraction and modification)
// - KISS port translator (port number translation)
// - PhilFlag processing (TASCO modem bug workaround)
// - Frame parsing and display functions
//
// ==============================================================================

// ==============================================================================
// LOGGER
// ==============================================================================

/// Thread-safe logger with multiple output targets
/// 
/// Supports logging to:
/// - Console (stdout)
/// - Log file (append mode)
/// - Both simultaneously
/// 
/// Log levels (syslog-style):
///   0 = EMERG, 1 = ALERT, 2 = CRIT, 3 = ERROR, 4 = WARN
///   5 = NOTICE, 6 = INFO, 7 = DEBUG, 8 = TRACE, 9 = VERBOSE
struct Logger {
    file: Option<Arc<Mutex<File>>>,  // Optional log file (thread-safe)
    log_level: u8,                   // Current log level threshold
    log_to_console: bool,            // Whether to also log to stdout
}

impl Logger {
    /// Create a new logger
    /// 
    /// Arguments:
    ///   logfile: Optional path to log file
    ///   log_level: Maximum level to log (0-9)
    ///   log_to_console: Also print to stdout
    fn new(
        logfile: Option<String>, 
        log_level: u8, 
        log_to_console: bool
    ) -> Result<Self, Box<dyn std::error::Error>> {
        
        // Open log file if specified
        let file = if let Some(path) = logfile {
            let f = OpenOptions::new()
                .create(true)      // Create if doesn't exist
                .append(true)      // Append to existing content
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
    
    /// Log a message at the specified level
    /// 
    /// Format: [timestamp] [LEVEL] message
    /// Example: [2025-12-24 18:30:00] [NOTICE] rax25kb starting
    fn log(&self, message: &str, level: u8) {
        // Skip if message level exceeds threshold
        if level > self.log_level { 
            return; 
        }
        
        // Get current timestamp
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        
        // Convert level number to string
        let level_str = match level {
            0 => "EMERG",    // System is unusable
            1 => "ALERT",    // Action must be taken immediately
            2 => "CRIT",     // Critical conditions
            3 => "ERROR",    // Error conditions
            4 => "WARN",     // Warning conditions
            5 => "NOTICE",   // Normal but significant
            6 => "INFO",     // Informational
            7 => "DEBUG",    // Debug-level messages
            8 => "TRACE",    // Trace execution
            9 => "VERBOSE",  // Very detailed output
            _ => "UNKNOWN",
        };
        
        // Format complete log line
        let log_line = format!("[{}] [{}] {}\n", timestamp, level_str, message);
        
        // Write to console if enabled
        if self.log_to_console { 
            print!("{}", log_line); 
        }
        
        // Write to file if configured
        if let Some(ref file) = self.file {
            if let Ok(mut f) = file.lock() {
                let _ = f.write_all(log_line.as_bytes());
            }
        }
    }
}

// ==============================================================================
// PCAP WRITER
// ==============================================================================

/// PCAP (Packet Capture) file writer for Wireshark analysis
/// 
/// Creates files compatible with Wireshark and other protocol analyzers.
/// Uses DLT_AX25_KISS (147) link type for AX.25 frames.
/// 
/// File format:
///   Global Header (24 bytes)
///   Packet Header 1 (16 bytes) + Packet Data 1
///   Packet Header 2 (16 bytes) + Packet Data 2
///   ...
struct PcapWriter {
    file: Arc<Mutex<File>>,  // Thread-safe file handle
}

impl PcapWriter {
    /// Create a new PCAP file and write the global header
    /// 
    /// PCAP Global Header:
    ///   magic_number:   4 bytes (0xa1b2c3d4 = little-endian)
    ///   version_major:  2 bytes (2)
    ///   version_minor:  2 bytes (4)
    ///   thiszone:       4 bytes (0 = GMT)
    ///   sigfigs:        4 bytes (0 = timestamp accuracy)
    ///   snaplen:        4 bytes (65535 = max packet length)
    ///   network:        4 bytes (147 = DLT_AX25_KISS)
    fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::create(path)?;
        
        // Write PCAP global header
        file.write_all(&0xa1b2c3d4u32.to_le_bytes())?; // Magic number (native byte order)
        file.write_all(&2u16.to_le_bytes())?;          // Version major = 2
        file.write_all(&4u16.to_le_bytes())?;          // Version minor = 4
        file.write_all(&0i32.to_le_bytes())?;          // Timezone = GMT
        file.write_all(&0u32.to_le_bytes())?;          // Timestamp accuracy = 0
        file.write_all(&65535u32.to_le_bytes())?;      // Snapshot length = max
        file.write_all(&147u32.to_le_bytes())?;        // Link type = AX.25 KISS
        
        Ok(PcapWriter { 
            file: Arc::new(Mutex::new(file)) 
        })
    }
    
    /// Write a packet to the PCAP file
    /// 
    /// PCAP Packet Header:
    ///   ts_sec:     4 bytes (timestamp seconds)
    ///   ts_usec:    4 bytes (timestamp microseconds)
    ///   incl_len:   4 bytes (captured packet length)
    ///   orig_len:   4 bytes (original packet length)
    /// 
    /// Followed by the actual packet data
    fn write_packet(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        // Get current time as Unix timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?;
        
        let mut file = self.file.lock().unwrap();
        
        // Write packet header
        file.write_all(&(now.as_secs() as u32).to_le_bytes())?;      // Seconds
        file.write_all(&(now.subsec_micros() as u32).to_le_bytes())?; // Microseconds
        file.write_all(&(data.len() as u32).to_le_bytes())?;          // Captured length
        file.write_all(&(data.len() as u32).to_le_bytes())?;          // Original length
        
        // Write packet data
        file.write_all(data)?;
        
        // Ensure data is written to disk
        file.flush()?;
        
        Ok(())
    }
}

// ==============================================================================
// KISS FRAME BUFFER
// ==============================================================================

/// Accumulates bytes and detects KISS frame boundaries
/// 
/// KISS frames are delimited by FEND (0xC0) bytes:
///   FEND [port+cmd] [data] FEND
/// 
/// This buffer handles:
/// - Partial frames arriving over multiple reads
/// - Multiple frames in a single read
/// - Frame boundary detection
struct KissFrameBuffer {
    buffer: Vec<u8>,  // Accumulated bytes
    in_frame: bool,   // Currently inside a frame
}

impl KissFrameBuffer {
    /// Create a new empty frame buffer
    fn new() -> Self {
        KissFrameBuffer {
            buffer: Vec::new(),
            in_frame: false,
        }
    }
    
    /// Add bytes to the buffer and extract complete frames
    /// 
    /// Returns a vector of complete KISS frames (including FEND delimiters)
    fn add_bytes(&mut self, data: &[u8]) -> Vec<Vec<u8>> {
        let mut frames = Vec::new();
        
        for &byte in data {
            if byte == KISS_FEND {
                if self.in_frame && !self.buffer.is_empty() {
                    // End of frame - add final FEND and extract
                    self.buffer.push(byte);
                    frames.push(self.buffer.clone());
                    self.buffer.clear();
                    self.in_frame = false;
                } else {
                    // Start of new frame
                    self.buffer.clear();
                    self.buffer.push(byte);
                    self.in_frame = true;
                }
            } else if self.in_frame {
                // Accumulate bytes inside frame
                self.buffer.push(byte);
            }
            // Bytes outside frames are discarded
        }
        
        frames
    }
}

// ==============================================================================
// KISS HELPER FUNCTIONS
// ==============================================================================

/// Extract KISS port number and command from a frame
/// 
/// Returns: (port_number, command, data_start_index)
/// 
/// Frame format: FEND [port+cmd] [data] FEND
/// Where port+cmd byte is: (port << 4) | command
///   port:    bits 4-7 (0-15)
///   command: bits 0-3 (0-15)
fn extract_kiss_info(frame: &[u8]) -> Option<(u8, u8, usize)> {
    // Validate frame has at least FEND + command byte
    if frame.len() < 2 || frame[0] != KISS_FEND {
        return None;
    }
    
    // Extract port and command from second byte
    let cmd_byte = frame[1];
    let port = (cmd_byte >> 4) & 0x0F;     // Upper 4 bits
    let command = cmd_byte & 0x0F;         // Lower 4 bits
    
    // Data starts at index 2
    Some((port, command, 2))
}

/// Modify the KISS port number in a frame
/// 
/// Takes a complete KISS frame and changes the port number in the
/// command byte while preserving the command.
/// 
/// Returns a new frame with the modified port number
fn modify_kiss_port(frame: &[u8], new_port: u8) -> Vec<u8> {
    // Validate frame format
    if frame.len() < 2 || frame[0] != KISS_FEND {
        return frame.to_vec();
    }
    
    let mut result = frame.to_vec();
    
    // Extract original command (lower 4 bits)
    let cmd_byte = frame[1];
    let command = cmd_byte & 0x0F;
    
    // Construct new command byte with new port
    result[1] = ((new_port & 0x0F) << 4) | command;
    
    result
}

// ==============================================================================
// KISS PORT TRANSLATOR
// ==============================================================================

/// Translates KISS frames between different port numbers
/// 
/// Used for:
/// - Serial-to-serial bridges with different port numbers
/// - Standard KISS to Extended KISS translation
/// - Routing frames between TNCs with different addressing
struct KissPortTranslator {
    source_port: u8,  // Only translate frames from this port
    dest_port: u8,    // Translate to this port
}

impl KissPortTranslator {
    /// Create a new translator
    fn new(source_port: u8, dest_port: u8) -> Self {
        KissPortTranslator { 
            source_port, 
            dest_port 
        }
    }
    
    /// Translate a frame if it matches the source port
    /// 
    /// Returns:
    ///   Some(frame) - Translated frame
    ///   None - Frame not for this port or no translation needed
    fn translate(&self, frame: &[u8]) -> Option<Vec<u8>> {
        // Extract port info from frame
        let (current_port, _command, _data_start) = extract_kiss_info(frame)?;
        
        // Only process frames from our source port
        if current_port != self.source_port {
            return None;
        }
        
        // Skip translation if ports are the same
        if self.source_port == self.dest_port {
            return None;
        }
        
        // Translate port number
        Some(modify_kiss_port(frame, self.dest_port))
    }
}

// ==============================================================================
// PHILFLAG PROCESSING
// ==============================================================================

/// Process frame with PhilFlag correction (Serial → TCP direction)
/// 
/// PhilFlag fixes KISS protocol bugs in TASCO modem chipsets and
/// Kenwood TS-2000 TNCs. These devices incorrectly handle FEND bytes
/// in the data stream.
/// 
/// This function escapes any unescaped FEND bytes in the middle of
/// the frame by converting them to FESC TFEND sequences.
fn process_frame_with_phil_flag(frame: &[u8]) -> Vec<u8> {
    if frame.len() < 2 { 
        return frame.to_vec(); 
    }
    
    let mut output = Vec::with_capacity(frame.len() * 2);
    
    // Keep first FEND as-is
    output.push(frame[0]);
    
    // Escape any FEND bytes in the middle of the frame
    for i in 1..frame.len()-1 {
        if frame[i] == KISS_FEND {
            // Convert FEND → FESC TFEND
            output.push(KISS_FESC);
            output.push(KISS_TFEND);
        } else {
            output.push(frame[i]);
        }
    }
    
    // Keep last byte (usually FEND) as-is
    if frame.len() > 1 { 
        output.push(frame[frame.len()-1]); 
    }
    
    output
}

/// Process data with PhilFlag correction (TCP → Serial direction)
/// 
/// Escapes 'C' and 'c' characters to prevent TASCO modems from
/// misinterpreting "TC0\n" sequences as commands.
/// 
/// This bug affects certain TNC firmwares that use "TC0\n" as a
/// special command sequence. Escaping these characters prevents
/// accidental triggering of this command.
fn process_phil_flag_tcp_to_serial(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(data.len() * 2);
    
    for &byte in data {
        if byte == 0x43 || byte == 0x63 {  // 'C' or 'c'
            // Escape by prepending FESC
            output.push(KISS_FESC);
            output.push(byte);
        } else {
            output.push(byte);
        }
    }
    
    output
}

// ==============================================================================
// KISS FRAME PARSING AND DISPLAY
// ==============================================================================

/// Parse and display a KISS frame
/// 
/// Shows:
/// - KISS port number and command type
/// - Frame length
/// - AX.25 frame details (if data frame)
/// 
/// Also writes to PCAP file if configured
fn parse_kiss_frame_static(
    data: &[u8], 
    direction: &str, 
    pcap_writer: &Option<Arc<PcapWriter>>,
    dump_ax25: bool
) {
    // Validate frame has proper FEND start
    if data.len() < 2 || data[0] != KISS_FEND { 
        return; 
    }
    
    // Find end FEND
    let end_pos = match data.iter().skip(1).position(|&b| b == KISS_FEND) {
        Some(pos) => pos + 1,
        None => return,
    };
    
    // Extract frame content (between FENDs)
    let frame_data = &data[1..end_pos];
    if frame_data.is_empty() { 
        return; 
    }
    
    // Parse command byte
    let cmd_byte = frame_data[0];
    let port = (cmd_byte >> 4) & 0x0F;
    let command = cmd_byte & 0x0F;
    
    // Decode command type
    let cmd_name = match command {
        0 => "Data",           // Data frame (most common)
        1 => "TXDELAY",        // Set transmit delay
        2 => "Persistence",    // Set persistence parameter
        3 => "SlotTime",       // Set slot time
        4 => "TXtail",         // Set transmit tail
        5 => "FullDuplex",     // Set full duplex mode
        6 => "SetHardware",    // Set hardware parameters
        15 => "Return",        // Exit KISS mode
        _ => "Unknown",
    };
    
    // Display KISS frame info
    println!("=== {} KISS Frame ===", direction);
    println!("  Port: {}, Command: {} ({})", port, command, cmd_name);
    println!("  Frame length: {} bytes", data.len());
    
    // Process data frames (command 0)
    if command == 0 && frame_data.len() > 1 {
        let ax25_data = &frame_data[1..];
        
        // Write to PCAP if configured
        if let Some(ref pcap) = pcap_writer {
            let _ = pcap.write_packet(ax25_data);
        }
        
        // Parse and display AX.25 frame if dump_ax25 is enabled
        if dump_ax25 {
            if let Some(ax25_frame) = AX25Frame::parse(ax25_data) {
                ax25_frame.print_summary();
            }
        }
    }
    
    println!();
}

/// Display a frame in hexadecimal dump format
/// 
/// Format:
///   00000000: 01 02 03 04 05 06 07 08  09 0a 0b 0c 0d 0e 0f 10  ................
///   00000010: 11 12 13 14 15 16 17 18  19 1a 1b 1c 1d 1e 1f 20  ............... 
fn dump_frame(data: &[u8], title: &str) {
    println!("=== {} ({} bytes) ===", title, data.len());
    
    for (i, chunk) in data.chunks(16).enumerate() {
        // Address
        print!("{:08x}: ", i * 16);
        
        // Hex dump (with separator at 8 bytes)
        for (j, byte) in chunk.iter().enumerate() {
            print!("{:02x}", byte);
            if j == 7 { 
                print!(" "); 
            }
            print!(" ");
        }
        
        // Padding for incomplete lines
        if chunk.len() < 16 {
            for j in chunk.len()..16 {
                if j == 8 { 
                    print!(" "); 
                }
                print!("   ");
            }
        }
        
        print!(" ");
        
        // ASCII dump (printable chars only)
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
// END OF PART 3
// ==============================================================================
// Continue with Part 4: Serial port manager and cross-connect manager// ==============================================================================
// MAIN.RS - PART 4 OF 5
// ==============================================================================
//
// This part contains:
// - SerialPortManager: Opens and manages multiple serial ports
// - CrossConnectManager: Creates and manages cross-connect connections
// - Connection handlers for:
//   * Serial-to-TCP (with KISS processing)
//   * Serial-to-Serial (with port translation)
//   * Raw copy mode (no KISS processing)
//
// Architecture:
//   CrossConnectManager spawns threads for each cross-connect.
//   Each serial-to-TCP connection gets 2 threads (bidirectional).
//   Serial-to-serial bridges get 2 threads (one per direction).
//
// ==============================================================================

// ==============================================================================
// SERIAL PORT MANAGER
// ==============================================================================

/// Manages multiple serial ports with thread-safe access
/// 
/// Each serial port is:
/// - Opened with specified configuration (baud, parity, etc.)
/// - Wrapped in Arc<Mutex<>> for thread-safe sharing
/// - Stored in a HashMap by port ID
struct SerialPortManager {
    ports: HashMap<String, Arc<Mutex<Box<dyn serialport::SerialPort>>>>,
}

impl SerialPortManager {
    /// Create a new empty manager
    fn new() -> Self {
        SerialPortManager { 
            ports: HashMap::new() 
        }
    }
    
    /// Open a serial port with the specified configuration
    /// 
    /// Configures:
    /// - Baud rate
    /// - Flow control (none, software, hardware, DTR/DSR)
    /// - Stop bits (1 or 2)
    /// - Parity (none, even, odd)
    /// - Timeout (100ms read timeout)
    fn open_port(
        &mut self, 
        config: &SerialPortConfig
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        // Create port builder with basic settings
        let mut port_builder = serialport::new(&config.device, config.baud_rate)
            .timeout(Duration::from_millis(100));
        
        // Configure flow control
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
                // DTR/DSR is Windows-specific
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
        
        // Configure stop bits
        port_builder = match config.stop_bits {
            StopBits::One => {
                port_builder.stop_bits(serialport::StopBits::One)
            }
            StopBits::Two => {
                port_builder.stop_bits(serialport::StopBits::Two)
            }
        };
        
        // Configure parity
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
        
        // Open the port
        let port = port_builder.open()?;
        
        // Store in HashMap wrapped for thread safety
        self.ports.insert(
            config.id.clone(), 
            Arc::new(Mutex::new(port))
        );
        
        Ok(())
    }
    
    /// Get a cloned Arc reference to a serial port by ID
    /// 
    /// Returns None if the port doesn't exist
    fn get_port(&self, id: &str) -> Option<Arc<Mutex<Box<dyn serialport::SerialPort>>>> {
        self.ports.get(id).map(|p| Arc::clone(p))
    }
}

// ==============================================================================
// CROSS-CONNECT MANAGER
// ==============================================================================

/// Manages all cross-connect connections
/// 
/// Responsibilities:
/// - Opens all serial ports
/// - Starts all cross-connect threads
/// - Manages connection lifecycle
struct CrossConnectManager {
    config: Arc<Config>,                         // Shared configuration
    serial_manager: Arc<Mutex<SerialPortManager>>, // Serial port manager
    logger: Arc<Logger>,                         // Shared logger
    pcap_writer: Option<Arc<PcapWriter>>,        // Optional PCAP capture
}

impl CrossConnectManager {
    /// Create a new manager and open all serial ports
    fn new(
        config: Config, 
        logger: Arc<Logger>, 
        pcap_writer: Option<Arc<PcapWriter>>
    ) -> Result<Self, Box<dyn std::error::Error>> {
        
        let mut serial_manager = SerialPortManager::new();
        
        // Open all configured serial ports
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
    
    /// Start all configured cross-connects
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
    
    /// Start a single cross-connect based on endpoint types
    fn start_cross_connect(
        &self, 
        cc: &CrossConnect
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        match (&cc.endpoint_a, &cc.endpoint_b) {
            // Serial to TCP (either direction)
            (CrossConnectEndpoint::SerialPort { port_id, kiss_port }, 
             CrossConnectEndpoint::TcpSocket { address, port }) |
            (CrossConnectEndpoint::TcpSocket { address, port },
             CrossConnectEndpoint::SerialPort { port_id, kiss_port }) => {
                self.start_serial_to_tcp(cc, port_id, *kiss_port, address, *port)?;
            }
            
            // Serial to Serial (bridging)
            (CrossConnectEndpoint::SerialPort { port_id: id_a, kiss_port: port_a },
             CrossConnectEndpoint::SerialPort { port_id: id_b, kiss_port: port_b }) => {
                self.start_serial_to_serial(cc, id_a, *port_a, id_b, *port_b)?;
            }
            
            // TCP to TCP (not supported)
            (CrossConnectEndpoint::TcpSocket { .. }, 
             CrossConnectEndpoint::TcpSocket { .. }) => {
                return Err("TCP to TCP cross-connects not supported".into());
            }
        }
        
        Ok(())
    }
    
    /// Start a serial-to-TCP cross-connect
    /// 
    /// Creates a TCP listener that:
    /// - Accepts client connections
    /// - Spawns a handler thread for each client
    /// - Filters frames by KISS port number
    /// - Applies PhilFlag if configured
    fn start_serial_to_tcp(
        &self, 
        cc: &CrossConnect, 
        serial_id: &str, 
        kiss_port: u8,
        tcp_address: &str, 
        tcp_port: u16
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        // Bind TCP listener
        let bind_address = format!("{}:{}", tcp_address, tcp_port);
        let listener = TcpListener::bind(&bind_address)?;
        
        self.logger.log(
            &format!("Cross-connect {} listening on {}", cc.id, bind_address), 
            5
        );
        
        // Clone Arc references for the spawned thread
        let serial_manager = Arc::clone(&self.serial_manager);
        let serial_id = serial_id.to_string();
        let cc_config = cc.clone();
        let logger = Arc::clone(&self.logger);
        let pcap_writer = self.pcap_writer.clone();
        
        // Spawn listener thread
        thread::spawn(move || {
            loop {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        logger.log(
                            &format!("Cross-connect {}: Client connected from {}", 
                                cc_config.id, addr), 
                            5
                        );
                        
                        // Get serial port reference
                        let serial_port = {
                            let mgr = serial_manager.lock().unwrap();
                            mgr.get_port(&serial_id)
                        };
                        
                        if let Some(port) = serial_port {
                            // Handle this connection
                            Self::handle_serial_tcp(
                                stream, 
                                port, 
                                kiss_port, 
                                &cc_config, 
                                &logger, 
                                &pcap_writer
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
    
    /// Handle a serial-to-TCP connection
    /// 
    /// Creates two threads:
    /// 1. Serial → TCP: Reads from serial, filters by KISS port, sends to TCP
    /// 2. TCP → Serial: Reads from TCP, modifies KISS port, sends to serial
    fn handle_serial_tcp(
        mut stream: TcpStream, 
        serial_port: Arc<Mutex<Box<dyn serialport::SerialPort>>>,
        kiss_port: u8, 
        cc_config: &CrossConnect, 
        logger: &Arc<Logger>,
        pcap_writer: &Option<Arc<PcapWriter>>
    ) {
        // Handle raw copy mode (no KISS processing)
        if cc_config.raw_copy {
            Self::handle_raw_copy(stream, serial_port, logger);
            return;
        }
        
        // Clone references for serial→TCP thread
        let serial_clone = Arc::clone(&serial_port);
        let mut read_stream = stream.try_clone()
            .expect("Failed to clone stream");
        let logger_clone = Arc::clone(logger);
        let cc_clone = cc_config.clone();
        let pcap_clone = pcap_writer.clone();
        
        // Spawn Serial → TCP thread
        let serial_to_tcp = thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            let mut frame_buffer = KissFrameBuffer::new();
            
            loop {
                // Read from serial port
                let mut port = serial_clone.lock().unwrap();
                match port.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        drop(port); // Release lock immediately
                        
                        // Extract complete KISS frames
                        let frames = frame_buffer.add_bytes(&buffer[..n]);
                        
                        for frame in frames {
                            // Check if frame is for our KISS port
                            if let Some((port_num, _, _)) = extract_kiss_info(&frame) {
                                if port_num == kiss_port {
                                    // Apply PhilFlag if configured
                                    let processed = if cc_clone.phil_flag {
                                        process_frame_with_phil_flag(&frame)
                                    } else { 
                                        frame 
                                    };
                                    
                                    // Display frame if configured
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
                                    
                                    // Send to TCP client
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
        
        // Main thread handles TCP → Serial
        let mut buffer = [0u8; 1024];
        let mut frame_buffer = KissFrameBuffer::new();
        
        loop {
            match stream.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let frames = frame_buffer.add_bytes(&buffer[..n]);
                    
                    for frame in frames {
                        // Modify KISS port number
                        let modified = modify_kiss_port(&frame, kiss_port);
                        
                        // Apply PhilFlag if configured
                        let processed = if cc_config.phil_flag {
                            process_phil_flag_tcp_to_serial(&modified)
                        } else { 
                            modified 
                        };
                        
                        // Display frame if configured
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
        
        // Wait for serial→TCP thread to finish
        drop(serial_to_tcp);
    }
    
    /// Start a serial-to-serial cross-connect (bridge)
    /// 
    /// Creates two threads:
    /// 1. Port A → Port B: Translates KISS port numbers
    /// 2. Port B → Port A: Translates KISS port numbers
    fn start_serial_to_serial(
        &self, 
        cc: &CrossConnect, 
        id_a: &str, 
        port_a: u8,
        id_b: &str, 
        port_b: u8
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        // Get both serial ports
        let serial_a = { 
            self.serial_manager.lock().unwrap().get_port(id_a) 
        }.ok_or(format!("Serial port {} not found", id_a))?;
        
        let serial_b = { 
            self.serial_manager.lock().unwrap().get_port(id_b) 
        }.ok_or(format!("Serial port {} not found", id_b))?;
        
        // Create bidirectional translators
        let translator_a_to_b = KissPortTranslator::new(port_a, port_b);
        let translator_b_to_a = KissPortTranslator::new(port_b, port_a);
        
        let _logger = Arc::clone(&self.logger);
        let pcap_a = self.pcap_writer.clone();
        let pcap_b = self.pcap_writer.clone();
        let cc_a = cc.clone();
        let cc_b = cc.clone();
        
        // Spawn Port A → Port B thread
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
                            // Translate KISS port number
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
        
        // Spawn Port B → Port A thread
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            let mut fb = KissFrameBuffer::new();
            
            loop {
                let mut port = serial_b.lock().unwrap();
                match port.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        drop(port);
                        
                        for frame in fb.add_bytes(&buf[..n]) {
                            // Translate KISS port number
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
    
    /// Handle raw copy mode (no KISS processing)
    /// 
    /// Creates two threads that just copy bytes bidirectionally
    /// with no frame processing. Useful for TNC configuration.
    fn handle_raw_copy(
        mut stream: TcpStream, 
        serial: Arc<Mutex<Box<dyn serialport::SerialPort>>>, 
        logger: &Arc<Logger>
    ) {
        let s = Arc::clone(&serial);
        let mut rs = stream.try_clone().unwrap();
        let l = Arc::clone(logger);
        
        // Serial → TCP thread
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
        
        // TCP → Serial loop (main thread)
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
// END OF PART 4
// ==============================================================================
// Continue with Part 5: Main function// ==============================================================================
// MAIN.RS - PART 5 OF 5
// ==============================================================================
//
// This part contains:
// - Main function
// - Command-line argument parsing
// - Configuration loading and validation
// - Startup sequence
// - Signal handling (Ctrl+C)
// - PID file creation
// - Logger initialization
// - PCAP writer initialization
// - Cross-connect manager initialization and startup
//
// ==============================================================================

/// Main entry point for rax25kb
/// 
/// Workflow:
/// 1. Setup signal handler for graceful shutdown
/// 2. Parse command-line arguments
/// 3. Load and validate configuration file
/// 4. Display startup information
/// 5. Write PID file (if configured)
/// 6. Initialize logger
/// 7. Initialize PCAP writer (if configured)
/// 8. Create and start cross-connect manager
/// 9. Enter main loop (sleep forever while threads do the work)
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ==================================================================
    // SIGNAL HANDLER
    // ==================================================================
    
    // Setup graceful shutdown on Ctrl+C (SIGINT/SIGTERM)
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("\nReceived SIGINT, shutting down gracefully...");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
        process::exit(0);
    })?;
    
    // ==================================================================
    // COMMAND-LINE ARGUMENT PARSING
    // ==================================================================
    
    let args: Vec<String> = std::env::args().collect();
    
    // Handle --help / -h
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("rax25kb v2.0.0 - Multi-Port Cross-Connect KISS Bridge");
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
    
    // Get configuration file path (-c option)
    let config_file = args.iter()
        .position(|a| a == "-c")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("rax25kb.cfg");
    
    // Check for quiet flag (-q)
    let quiet = args.iter().any(|a| a == "-q" || a == "--quiet");
    
    // ==================================================================
    // LOAD CONFIGURATION
    // ==================================================================
    
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
    
    // ==================================================================
    // VALIDATE CONFIGURATION
    // ==================================================================
    
    // Ensure at least one serial port is configured
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
    
    // Ensure at least one cross-connect is configured
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
    
    // ==================================================================
    // DISPLAY STARTUP INFORMATION
    // ==================================================================
    
    if !quiet && !config.quiet_startup {
        println!("rax25kb v2.0.0 - Multi-Port Cross-Connect KISS Bridge");
        println!("======================================================");
        println!();
        
        // Display serial ports
        println!("Serial ports configured: {}", config.serial_ports.len());
        for (id, port) in &config.serial_ports {
            println!("  [{}] {} @ {} baud", id, port.device, port.baud_rate);
            
            // Show non-default settings
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
        
        // Display cross-connects
        println!("Cross-connects configured: {}", config.cross_connects.len());
        for cc in &config.cross_connects {
            println!("  [{}] {:?} <-> {:?}", 
                cc.id, cc.endpoint_a, cc.endpoint_b);
            
            // Show enabled features
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
    
    // ==================================================================
    // WRITE PID FILE
    // ==================================================================
    
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
    
    // ==================================================================
    // INITIALIZE LOGGER
    // ==================================================================
    
    let logger = Arc::new(Logger::new(
        config.logfile.clone(),
        config.log_level,
        config.log_to_console,
    )?);
    
    logger.log("rax25kb v2.0.0 starting", 5);
    logger.log(&format!("Configuration loaded from: {}", config_file), 6);
    logger.log(&format!("Serial ports: {}", config.serial_ports.len()), 6);
    logger.log(&format!("Cross-connects: {}", config.cross_connects.len()), 6);
    logger.log(&format!("Log level: {}", config.log_level), 6);
    
    // ==================================================================
    // INITIALIZE PCAP WRITER
    // ==================================================================
    
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
    
    // ==================================================================
    // CREATE CROSS-CONNECT MANAGER
    // ==================================================================
    
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
    
    // ==================================================================
    // START ALL CROSS-CONNECTS
    // ==================================================================
    
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
    
    // ==================================================================
    // MAIN LOOP
    // ==================================================================
    
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
    
    // Main loop - just sleep forever
    // All the work is done by the spawned threads
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

// ==============================================================================
// END OF PART 5
// ==============================================================================
//
// To assemble complete main.rs:
//
//   cat src/main_part1.rs \
//       src/main_part2.rs \
//       src/main_part3.rs \
//       src/main_part4.rs \
//       src/main_part5.rs > src/main.rs
//
// Then build:
//
//   cargo build --release
//
// The compiled binary will be at:
//
//   target/release/rax25kb
//
// ==============================================================================