// rax25kb - AX.25 KISS Bridge
//
// Copyright (C) 2025-2026 Kris Kirby, KE4AHR
//
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rax25kb main source
// Version: 1.7.3 
//
// Version: 1.7.3 also carried Version 1.7.0 in above text
// until updated.
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

use std::collections::{HashMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serialport::SerialPort;

const KISS_FEND: u8 = 0xC0;
const KISS_FESC: u8 = 0xDB;
const KISS_TFEND: u8 = 0xDC;
#[allow(dead_code)]
const KISS_TFESC: u8 = 0xDD;

#[derive(Debug, Clone)]
#[allow(dead_code)] // Some fields used conditionally based on configuration
struct CrossConnect {
    id: String,
    serial_port: String,
    baud_rate: u32,
    flow_control: FlowControl,
    stop_bits: StopBits,
    data_bits: DataBits,
    parity: Parity,
    tcp_address: String,
    tcp_port: u16,
    tcp_mode: TcpMode,
    tcp_server_address: Option<String>,
    tcp_server_port: Option<u16>,
    kiss_port: u8,
    kiss_chan: i32,              // -1 = all channels, 0-15 = specific channel
    kiss_copy: bool,             // Enable KISSCOPY (broadcast between clients)
    xkiss_mode: bool,
    xkiss_port: Option<u8>,
    xkiss_checksum: bool,
    xkiss_polling: bool,
    xkiss_poll_timer_ms: u64,
    xkiss_rx_buffer_size: usize,
    serial_to_serial: Option<String>,
    tcp_to_tcp_dangerous: bool,
    tcp_to_tcp_also_dangerous: bool,
    phil_flag: bool,
    dump_frames: bool,
    parse_kiss: bool,
    dump_ax25: bool,
    raw_copy: bool,
    reframe_large_packets: bool,
    is_primary_port: bool,       // True if this is KISS port 0 (controls serial params)
    agw_port: u8,                // AGW port number (0-255, default: 0)
    agw_enable: bool,            // Enable AGW for this cross-connect (default: false)
}

#[derive(Debug, Clone)]
struct Config {
    cross_connects: Vec<CrossConnect>,
    log_level: u8,
    logfile: Option<String>,
    pidfile: Option<String>,
    log_to_console: bool,
    log_to_file_only: bool,
    quiet_startup: bool,
    pcap_file: Option<String>,
    max_tcp_clients: usize,      // Global max TCP clients per port (default 3)
    agw_server_enable: bool,     // Enable AGW server (default: false)
    agw_server_address: String,  // AGW bind address (default: "0.0.0.0")
    agw_server_port: u16,        // AGW port (default: 8000)
    agw_max_clients: usize,      // Max AGW clients (default: 3)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TcpMode {
    Server,
    Client,
    None,
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum DataBits {
    Seven,
    Eight,
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
        
        let mut cross_connects = Vec::new();
        let mut cc_map: HashMap<String, HashMap<String, String>> = HashMap::new();
        
        for (key, value) in &config_map {
            if key.starts_with("cross_connect") && key.len() >= 17 {
                let cc_id = &key[..17];
                let param = &key[18..];
                cc_map.entry(cc_id.to_string())
                    .or_insert_with(HashMap::new)
                    .insert(param.to_string(), value.clone());
            }
        }
        
        if cc_map.is_empty() {
            let default_cc = CrossConnect {
                id: "cross_connect0000".to_string(),
                serial_port: config_map.get("serial_port")
                    .ok_or("Missing required config: serial_port")?
                    .clone(),
                baud_rate: config_map.get("baud_rate")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(9600),
                flow_control: parse_flow_control(config_map.get("flow_control")),
                stop_bits: parse_stop_bits(config_map.get("stop_bits")),
                data_bits: parse_data_bits(config_map.get("data_bits")),
                parity: parse_parity(config_map.get("parity")),
                tcp_address: config_map.get("tcp_address")
                    .cloned()
                    .unwrap_or_else(|| "0.0.0.0".to_string()),
                tcp_port: config_map.get("tcp_port")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(8001),
                tcp_mode: parse_tcp_mode(config_map.get("tcp_mode")),
                tcp_server_address: config_map.get("tcp_server_address").cloned(),
                tcp_server_port: config_map.get("tcp_server_port")
                    .and_then(|v| v.parse().ok()),
                kiss_port: 0,
                kiss_chan: -1,  // Default: all channels
                kiss_copy: false,  // Default: no KISSCOPY
                xkiss_mode: false,
                xkiss_port: None,
                xkiss_checksum: false,
                xkiss_polling: false,
                xkiss_poll_timer_ms: 100,
                xkiss_rx_buffer_size: config_map.get("xkiss_rx_buffer_size")
                    .and_then(|v| v.parse().ok())
                    .map(|s: usize| s.clamp(4096, 1048576))
                    .unwrap_or(16384),
                serial_to_serial: None,
                tcp_to_tcp_dangerous: parse_bool(config_map.get("tcp_to_tcp_dangerous")),
                tcp_to_tcp_also_dangerous: parse_bool(config_map.get("tcp_to_tcp_also_dangerous")),
                phil_flag: parse_bool(config_map.get("phil_flag")),
                dump_frames: parse_bool(config_map.get("dump")),
                parse_kiss: parse_bool(config_map.get("parse_kiss")),
                dump_ax25: parse_bool(config_map.get("dump_ax25")),
                raw_copy: parse_bool(config_map.get("raw_copy")),
                reframe_large_packets: parse_bool(config_map.get("reframe_large_packets")),
                is_primary_port: true,  // Legacy mode is always primary
                agw_port: 0,            // Default AGW port
                agw_enable: false,      // AGW disabled by default in legacy mode
            };
            cross_connects.push(default_cc);
        } else {
            for (cc_id, params) in cc_map {
                let cc = CrossConnect {
                    id: cc_id.clone(),
                    serial_port: params.get("serial_port")
                        .ok_or(format!("Missing serial_port for {}", cc_id))?
                        .clone(),
                    baud_rate: params.get("baud_rate")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(9600),
                    flow_control: parse_flow_control(params.get("flow_control")),
                    stop_bits: parse_stop_bits(params.get("stop_bits")),
                    data_bits: parse_data_bits(params.get("data_bits")),
                    parity: parse_parity(params.get("parity")),
                    tcp_address: params.get("tcp_address")
                        .cloned()
                        .unwrap_or_else(|| "0.0.0.0".to_string()),
                    tcp_port: params.get("tcp_port")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(8001),
                    tcp_mode: parse_tcp_mode(params.get("tcp_mode")),
                    tcp_server_address: params.get("tcp_server_address").cloned(),
                    tcp_server_port: params.get("tcp_server_port")
                        .and_then(|v| v.parse().ok()),
                    kiss_port: params.get("kiss_port")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0),
                    kiss_chan: params.get("kiss_chan")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(-1),  // Default: all channels
                    kiss_copy: parse_bool(params.get("kiss_copy")),
                    xkiss_mode: parse_bool(params.get("xkiss_mode")),
                    xkiss_port: params.get("xkiss_port")
                        .and_then(|v| v.parse().ok()),
                    xkiss_checksum: parse_bool(params.get("xkiss_checksum")),
                    xkiss_polling: parse_bool(params.get("xkiss_polling")),
                    xkiss_poll_timer_ms: params.get("xkiss_poll_timer_ms")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(100),
                    xkiss_rx_buffer_size: params.get("xkiss_rx_buffer_size")
                        .and_then(|v| v.parse().ok())
                        .map(|s: usize| s.clamp(4096, 1048576))
                        .unwrap_or(16384),
                    serial_to_serial: params.get("serial_to_serial").cloned(),
                    tcp_to_tcp_dangerous: parse_bool(params.get("tcp_to_tcp_dangerous")),
                    tcp_to_tcp_also_dangerous: parse_bool(params.get("tcp_to_tcp_also_dangerous")),
                    phil_flag: parse_bool(params.get("phil_flag")),
                    dump_frames: parse_bool(params.get("dump")),
                    parse_kiss: parse_bool(params.get("parse_kiss")),
                    dump_ax25: parse_bool(params.get("dump_ax25")),
                    raw_copy: parse_bool(params.get("raw_copy")),
                    reframe_large_packets: parse_bool(params.get("reframe_large_packets")),
                    is_primary_port: false,  // Will be set later
                    agw_port: params.get("agw_port")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0),       // Default AGW port 0
                    agw_enable: parse_bool(params.get("agw_enable")),
                };
                cross_connects.push(cc);
            }
        }
        
        // Identify primary ports (KISS port 0 for each serial port)
        let mut serial_port_groups: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, cc) in cross_connects.iter().enumerate() {
            serial_port_groups.entry(cc.serial_port.clone())
                .or_insert_with(Vec::new)
                .push(idx);
        }
        
        for (_serial_port, indices) in serial_port_groups {
            // Find KISS port 0
            if let Some(&primary_idx) = indices.iter().find(|&&idx| cross_connects[idx].kiss_port == 0) {
                cross_connects[primary_idx].is_primary_port = true;
            } else if let Some(&first_idx) = indices.first() {
                // If no port 0, make first one primary
                cross_connects[first_idx].is_primary_port = true;
            }
        }
        
        let log_level = config_map.get("log_level")
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        
        let logfile = config_map.get("logfile").cloned();
        let pidfile = config_map.get("pidfile").cloned();
        
        let log_to_console = !config_map.get("log_to_console")
            .map(|v| matches!(v.to_lowercase().as_str(), "0" | "false" | "no"))
            .unwrap_or(false);
        
        let log_to_file_only = parse_bool(config_map.get("log_to_file_only"));
        let quiet_startup = parse_bool(config_map.get("quiet_startup"));
        let pcap_file = config_map.get("pcap_file").cloned();
        
        let max_tcp_clients = config_map.get("max_tcp_clients")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(3);  // Default 3 clients
        
        let agw_server_enable = parse_bool(config_map.get("agw_server_enable"));
        let agw_server_address = config_map.get("agw_server_address")
            .cloned()
            .unwrap_or_else(|| "0.0.0.0".to_string());
        let agw_server_port = config_map.get("agw_server_port")
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8000);  // Default AGW port
        let agw_max_clients = config_map.get("agw_max_clients")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(3);  // Default 3 AGW clients
        
        Ok(Config {
            cross_connects,
            log_level,
            logfile,
            pidfile,
            log_to_console,
            log_to_file_only,
            quiet_startup,
            pcap_file,
            max_tcp_clients,
            agw_server_enable,
            agw_server_address,
            agw_server_port,
            agw_max_clients,
        })
    }

    fn apply_cli_overrides(&mut self, args: &[String]) {
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-D" | "--device" if i + 1 < args.len() => {
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.serial_port = args[i + 1].clone();
                    }
                    i += 2;
                }
                "-b" | "--baud-rate" if i + 1 < args.len() => {
                    if let Ok(rate) = args[i + 1].parse() {
                        if let Some(cc) = self.cross_connects.get_mut(0) {
                            cc.baud_rate = rate;
                        }
                    }
                    i += 2;
                }
                "-s" | "--stop-bits" if i + 1 < args.len() => {
                    let sb = match args[i + 1].as_str() {
                        "1" => StopBits::One,
                        "2" => StopBits::Two,
                        _ => StopBits::One,
                    };
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.stop_bits = sb;
                    }
                    i += 2;
                }
                "-Q" | "--parity" if i + 1 < args.len() => {
                    let p = match args[i + 1].to_lowercase().as_str() {
                        "n" | "none" => Parity::None,
                        "e" | "even" => Parity::Even,
                        "o" | "odd" => Parity::Odd,
                        _ => Parity::None,
                    };
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.parity = p;
                    }
                    i += 2;
                }
                "-x" | "--xon-xoff" => {
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.flow_control = FlowControl::Software;
                    }
                    i += 1;
                }
                "-H" | "--rts-cts" => {
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.flow_control = FlowControl::Hardware;
                    }
                    i += 1;
                }
                "--dtr-dsr" => {
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.flow_control = FlowControl::DtrDsr;
                    }
                    i += 1;
                }
                "-N" | "--none" => {
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.flow_control = FlowControl::None;
                    }
                    i += 1;
                }
                "-I" | "--address" if i + 1 < args.len() => {
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.tcp_address = args[i + 1].clone();
                    }
                    i += 2;
                }
                "-p" | "--port" if i + 1 < args.len() => {
                    if let Ok(port) = args[i + 1].parse() {
                        if let Some(cc) = self.cross_connects.get_mut(0) {
                            cc.tcp_port = port;
                        }
                    }
                    i += 2;
                }
                "-d" | "--dump" => {
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.dump_frames = true;
                    }
                    i += 1;
                }
                "-k" | "--kiss" => {
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.parse_kiss = true;
                    }
                    i += 1;
                }
                "-a" | "--ax25" => {
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.dump_ax25 = true;
                    }
                    i += 1;
                }
                "-n" | "--phil" => {
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.phil_flag = true;
                    }
                    i += 1;
                }
                "-R" | "--raw-copy" => {
                    if let Some(cc) = self.cross_connects.get_mut(0) {
                        cc.raw_copy = true;
                    }
                    i += 1;
                }
                "-l" | "--logfile" if i + 1 < args.len() => {
                    self.logfile = Some(args[i + 1].clone());
                    i += 2;
                }
                "-L" | "--log-level" if i + 1 < args.len() => {
                    if let Ok(level) = args[i + 1].parse() {
                        self.log_level = level;
                    }
                    i += 2;
                }
                "--console-only" => {
                    self.log_to_file_only = false;
                    self.log_to_console = true;
                    i += 1;
                }
                "--no-console" => {
                    self.log_to_console = false;
                    self.log_to_file_only = true;
                    i += 1;
                }
                "-P" | "--pidfile" if i + 1 < args.len() => {
                    self.pidfile = Some(args[i + 1].clone());
                    i += 2;
                }
                "--pcap" if i + 1 < args.len() => {
                    self.pcap_file = Some(args[i + 1].clone());
                    i += 2;
                }
                "-q" | "--quiet" => {
                    self.quiet_startup = true;
                    i += 1;
                }
                "-c" => {
                    i += 2;
                }
                _ => i += 1,
            }
        }
    }
}

fn parse_flow_control(opt: Option<&String>) -> FlowControl {
    opt.and_then(|v| match v.to_lowercase().as_str() {
        "software" | "xon" | "xonxoff" | "xon-xoff" => Some(FlowControl::Software),
        "hardware" | "rtscts" | "rts-cts" | "rts/cts" => Some(FlowControl::Hardware),
        "dtrdsr" | "dtr-dsr" | "dtr/dsr" => Some(FlowControl::DtrDsr),
        "none" | "off" | "no" => Some(FlowControl::None),
        _ => None
    }).unwrap_or(FlowControl::None)
}

fn parse_stop_bits(opt: Option<&String>) -> StopBits {
    opt.and_then(|v| match v.as_str() {
        "1" | "one" => Some(StopBits::One),
        "2" | "two" => Some(StopBits::Two),
        _ => None
    }).unwrap_or(StopBits::One)
}

fn parse_parity(opt: Option<&String>) -> Parity {
    opt.and_then(|v| match v.to_lowercase().as_str() {
        "none" | "n" | "no" => Some(Parity::None),
        "odd" | "o" => Some(Parity::Odd),
        "even" | "e" => Some(Parity::Even),
        _ => None
    }).unwrap_or(Parity::None)
}

fn parse_data_bits(opt: Option<&String>) -> DataBits {
    opt.and_then(|v| match v.as_str() {
        "7" | "seven" => Some(DataBits::Seven),
        "8" | "eight" => Some(DataBits::Eight),
        _ => None
    }).unwrap_or(DataBits::Eight)
}

fn parse_tcp_mode(opt: Option<&String>) -> TcpMode {
    opt.and_then(|v| match v.to_lowercase().as_str() {
        "server" => Some(TcpMode::Server),
        "client" => Some(TcpMode::Client),
        "none" => Some(TcpMode::None),
        _ => None
    }).unwrap_or(TcpMode::Server)
}

fn parse_bool(opt: Option<&String>) -> bool {
    opt.and_then(|v| match v.to_lowercase().as_str() {
        "1" | "true" | "yes" => Some(true),
        _ => Some(false)
    }).unwrap_or(false)
}

struct Logger {
    file: Option<Arc<Mutex<File>>>,
    log_level: u8,
    log_to_console: bool,
}

impl Logger {
    fn new(logfile: Option<String>, log_level: u8, log_to_console: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let file = if let Some(path) = logfile {
            Some(Arc::new(Mutex::new(OpenOptions::new().create(true).append(true).open(path)?)))
        } else {
            None
        };
        Ok(Logger { file, log_level, log_to_console })
    }

    fn log(&self, message: &str, level: u8) {
        if level > self.log_level { return; }
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
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
        let log_line = format!("[{}] [{}] {}", timestamp, level_str, message);
        if self.log_to_console { println!("{}", log_line); }
        if let Some(ref file) = self.file {
            if let Ok(mut f) = file.lock() {
                let _ = writeln!(f, "{}", log_line);
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
        let magic: u32 = 0xa1b2c3d4;
        let version_major: u16 = 2;
        let version_minor: u16 = 4;
        let thiszone: i32 = 0;
        let sigfigs: u32 = 0;
        let snaplen: u32 = 65535;
        let network: u32 = 3;
        file.write_all(&magic.to_le_bytes())?;
        file.write_all(&version_major.to_le_bytes())?;
        file.write_all(&version_minor.to_le_bytes())?;
        file.write_all(&thiszone.to_le_bytes())?;
        file.write_all(&sigfigs.to_le_bytes())?;
        file.write_all(&snaplen.to_le_bytes())?;
        file.write_all(&network.to_le_bytes())?;
        Ok(PcapWriter { file: Arc::new(Mutex::new(file)) })
    }

    fn write_packet(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
        let ts_sec = now.as_secs() as u32;
        let ts_usec = now.subsec_micros();
        let incl_len = data.len() as u32;
        let orig_len = data.len() as u32;
        if let Ok(mut file) = self.file.lock() {
            file.write_all(&ts_sec.to_le_bytes())?;
            file.write_all(&ts_usec.to_le_bytes())?;
            file.write_all(&incl_len.to_le_bytes())?;
            file.write_all(&orig_len.to_le_bytes())?;
            file.write_all(data)?;
        }
        Ok(())
    }
}

struct XkissRxBuffer {
    buffer: VecDeque<Vec<u8>>,
    max_size: usize,
    current_size: usize,
}

impl XkissRxBuffer {
    fn new(max_size: usize) -> Self {
        XkissRxBuffer {
            buffer: VecDeque::new(),
            max_size,
            current_size: 0,
        }
    }

    fn push(&mut self, packet: Vec<u8>) -> Result<(), String> {
        let packet_size = packet.len();
        if self.current_size + packet_size > self.max_size {
            return Err(format!("Buffer full: {} + {} > {}", 
                self.current_size, packet_size, self.max_size));
        }
        self.current_size += packet_size;
        self.buffer.push_back(packet);
        Ok(())
    }

    fn pop(&mut self) -> Option<Vec<u8>> {
        if let Some(packet) = self.buffer.pop_front() {
            self.current_size = self.current_size.saturating_sub(packet.len());
            Some(packet)
        } else {
            None
        }
    }

    fn poll_flush(&mut self) -> Vec<Vec<u8>> {
        let mut packets = Vec::new();
        while let Some(packet) = self.pop() {
            packets.push(packet);
        }
        packets
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }

    #[allow(dead_code)] // Utility method for future use
    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    #[allow(dead_code)] // Utility method for future use
    fn clear(&mut self) {
        self.buffer.clear();
        self.current_size = 0;
    }
}

struct TcpClientInfo {
    stream: TcpStream,
    #[allow(dead_code)]  // Used for connection tracking/debugging
    connected_at: std::time::SystemTime,
}

// AGW (AGWPE) Protocol Structures
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct AgwHeader {
    port: u32,           // Radio port number (little-endian)
    reserved1: u32,      // Always 0
    kind: u8,            // Data kind
    reserved2: u8,       // Always 0
    pid: u8,             // Protocol ID
    reserved3: u8,       // Always 0
    call_from: [u8; 10], // From callsign
    call_to: [u8; 10],   // To callsign
    data_len: u32,       // Data length (little-endian)
}

impl AgwHeader {
    const SIZE: usize = 36;
    
    fn new() -> Self {
        AgwHeader {
            port: 0,
            reserved1: 0,
            kind: 0,
            reserved2: 0,
            pid: 0,
            reserved3: 0,
            call_from: [0; 10],
            call_to: [0; 10],
            data_len: 0,
        }
    }
    
    fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        
        let mut header = AgwHeader::new();
        header.port = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        header.kind = data[8];
        header.pid = data[10];
        header.call_from.copy_from_slice(&data[12..22]);
        header.call_to.copy_from_slice(&data[22..32]);
        header.data_len = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);
        
        Some(header)
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![0u8; Self::SIZE];
        buf[0..4].copy_from_slice(&self.port.to_le_bytes());
        buf[4..8].copy_from_slice(&self.reserved1.to_le_bytes());
        buf[8] = self.kind;
        buf[9] = self.reserved2;
        buf[10] = self.pid;
        buf[11] = self.reserved3;
        buf[12..22].copy_from_slice(&self.call_from);
        buf[22..32].copy_from_slice(&self.call_to);
        buf[32..36].copy_from_slice(&self.data_len.to_le_bytes());
        buf
    }
}

struct AgwClientInfo {
    stream: TcpStream,
    #[allow(dead_code)] // Reserved for future use (connection tracking, stats)
    connected_at: std::time::SystemTime,
    registered_call: Option<String>,  // Registered callsign
    monitor_enabled: bool,             // Monitor mode flag
}

struct CrossConnectBridge {
    config: CrossConnect,
    serial_port: Arc<Mutex<Box<dyn SerialPort>>>,
    tcp_clients: Arc<Mutex<Vec<Option<TcpClientInfo>>>>,  // Multiple clients
    max_clients: usize,                                     // From global config
    serial_peer: Option<Arc<Mutex<Box<dyn SerialPort>>>>,
    xkiss_rx_buffer: Arc<Mutex<XkissRxBuffer>>,
    logger: Arc<Logger>,
    pcap_writer: Option<Arc<PcapWriter>>,
    agw_clients: Arc<Mutex<Vec<Option<AgwClientInfo>>>>,  // AGW clients
    agw_enabled: bool,                                      // AGW enabled for this bridge
}

impl CrossConnectBridge {
    fn new(config: CrossConnect, max_clients: usize, 
           shared_serial: Option<Arc<Mutex<Box<dyn SerialPort>>>>,
           logger: Arc<Logger>, pcap_writer: Option<Arc<PcapWriter>>) 
        -> Result<Self, Box<dyn std::error::Error>> {
        
        let serial_port = if config.is_primary_port {
            // Primary port: Open new serial port with configured parameters
            // KISS specification requires 8N1 (8 data bits, no parity, 1 stop bit)
            // When KISS or XKISS is enabled, enforce 8N1 regardless of config
            let (data_bits, parity, stop_bits) = if !config.raw_copy {
                // KISS/XKISS mode: force 8N1
                (DataBits::Eight, Parity::None, StopBits::One)
            } else {
                // Raw copy mode: use configured values
                (config.data_bits, config.parity, config.stop_bits)
            };
            
            let port = serialport::new(&config.serial_port, config.baud_rate)
                .timeout(Duration::from_millis(100))
                .data_bits(match data_bits {
                    DataBits::Seven => serialport::DataBits::Seven,
                    DataBits::Eight => serialport::DataBits::Eight,
                })
                .stop_bits(match stop_bits {
                    StopBits::One => serialport::StopBits::One,
                    StopBits::Two => serialport::StopBits::Two,
                })
                .parity(match parity {
                    Parity::None => serialport::Parity::None,
                    Parity::Odd => serialport::Parity::Odd,
                    Parity::Even => serialport::Parity::Even,
                })
                .flow_control(match config.flow_control {
                    FlowControl::None => serialport::FlowControl::None,
                    FlowControl::Software => serialport::FlowControl::Software,
                    FlowControl::Hardware => serialport::FlowControl::Hardware,
                    FlowControl::DtrDsr => serialport::FlowControl::Hardware,
                })
                .open()?;
            
            Arc::new(Mutex::new(port))
        } else {
            // Secondary port: Use shared serial port from KISS port 0
            shared_serial.expect("Secondary port requires shared serial handle")
        };
        
        let xkiss_buffer = XkissRxBuffer::new(config.xkiss_rx_buffer_size);
        
        // Initialize client vector with None values
        let mut clients = Vec::with_capacity(max_clients);
        for _ in 0..max_clients {
            clients.push(None);
        }
        
        let agw_enabled = config.agw_enable;  // Save before moving config
        
        Ok(CrossConnectBridge {
            config,
            serial_port,
            tcp_clients: Arc::new(Mutex::new(clients)),
            max_clients,
            serial_peer: None,
            xkiss_rx_buffer: Arc::new(Mutex::new(xkiss_buffer)),
            logger,
            pcap_writer,
            agw_clients: Arc::new(Mutex::new((0..max_clients).map(|_| None).collect())),
            agw_enabled,
        })
    }

    #[allow(dead_code)] // API method for serial-to-serial peer setup
    fn set_serial_peer(&mut self, peer_port: Arc<Mutex<Box<dyn SerialPort>>>) {
        self.serial_peer = Some(peer_port);
    }

    #[allow(dead_code)] // Instance method kept for potential future use
    fn translate_kiss_port(&self, data: &[u8], from_xkiss: bool, to_xkiss: bool) -> Vec<u8> {
        if data.is_empty() || (!from_xkiss && !to_xkiss) {
            return data.to_vec();
        }

        let mut result = Vec::with_capacity(data.len());
        
        if from_xkiss && !to_xkiss {
            if let Some(_xkiss_port) = self.config.xkiss_port {
                for &byte in data {
                    if byte == KISS_FEND {
                        result.push(byte);
                    } else if result.len() == 1 {
                        let cmd_type = byte & 0x0F;
                        let new_byte = (self.config.kiss_port << 4) | cmd_type;
                        result.push(new_byte);
                    } else {
                        result.push(byte);
                    }
                }
            } else {
                result = data.to_vec();
            }
        } else if !from_xkiss && to_xkiss {
            if let Some(xkiss_port) = self.config.xkiss_port {
                for &byte in data {
                    if byte == KISS_FEND {
                        result.push(byte);
                    } else if result.len() == 1 {
                        let cmd_type = byte & 0x0F;
                        let new_byte = (xkiss_port << 4) | cmd_type;
                        result.push(new_byte);
                    } else {
                        result.push(byte);
                    }
                }
            } else {
                result = data.to_vec();
            }
        } else {
            result = data.to_vec();
        }
        
        result
    }

    fn start_tcp_client(&self) -> Result<(), Box<dyn std::error::Error>> {
        let server_addr = self.config.tcp_server_address.as_ref()
            .ok_or("Missing tcp_server_address for client mode")?;
        let server_port = self.config.tcp_server_port
            .ok_or("Missing tcp_server_port for client mode")?;
        let connect_address = format!("{}:{}", server_addr, server_port);
        
        self.logger.log(&format!("[{}] TCP client connecting to {}", 
            self.config.id, connect_address), 5);
        
        let tcp_clients = self.tcp_clients.clone();
        let serial = self.serial_port.clone();
        let serial_peer = self.serial_peer.clone();
        let config = self.config.clone();
        let logger = self.logger.clone();
        let connect_addr_clone = connect_address.clone();
        
        thread::spawn(move || {
            let mut reconnect_delay = Duration::from_secs(1);
            let max_reconnect_delay = Duration::from_secs(60);
            
            loop {
                match TcpStream::connect(&connect_addr_clone) {
                    Ok(mut stream) => {
                        logger.log(&format!("[{}] Connected to {}", config.id, connect_addr_clone), 5);
                        reconnect_delay = Duration::from_secs(1);
                        
                        {
                            let mut clients = tcp_clients.lock().unwrap();
                            clients[0] = Some(TcpClientInfo {
                                stream: stream.try_clone().unwrap(),
                                connected_at: std::time::SystemTime::now(),
                            });
                        }
                        
                        let tcp_clients_clone = tcp_clients.clone();
                        let serial_clone = serial.clone();
                        let serial_peer_clone = serial_peer.clone();
                        let config_clone = config.clone();
                        let logger_clone = logger.clone();
                        
                        let mut buffer = vec![0u8; 4096];
                        loop {
                            match stream.read(&mut buffer) {
                                Ok(0) => {
                                    logger_clone.log(&format!("[{}] Server disconnected", config_clone.id), 6);
                                    let mut clients = tcp_clients_clone.lock().unwrap();
                                    clients[0] = None;
                                    break;
                                }
                                Ok(n) => {
                                    let data = &buffer[..n];
                                    
                                    if config_clone.tcp_to_tcp_dangerous {
                                        if !config_clone.tcp_to_tcp_also_dangerous && !is_kiss_packet(data) {
                                            logger_clone.log(&format!("[{}] Non-KISS packet rejected in TCP-to-TCP mode", 
                                                config_clone.id), 4);
                                            continue;
                                        }
                                    }
                                    
                                    if config_clone.dump_frames {
                                        logger_clone.log(&format!("[{}] TCP->Serial ({} bytes): {:02x?}", 
                                            config_clone.id, n, data), 7);
                                    }
                                    
                                    let processed = if config_clone.raw_copy {
                                        data.to_vec()
                                    } else if config_clone.phil_flag {
                                        process_phil_flag_tcp_to_serial(data)
                                    } else {
                                        data.to_vec()
                                    };
                                    
                                    if let Some(ref peer) = serial_peer_clone {
                                        let translated = Self::translate_kiss_port_static(
                                            &processed, &config_clone, false, config_clone.xkiss_mode);
                                        if let Ok(mut port) = peer.lock() {
                                            let _ = port.write_all(&translated);
                                        }
                                    } else {
                                        if let Ok(mut port) = serial_clone.lock() {
                                            let _ = port.write_all(&processed);
                                        }
                                    }
                                }
                                Err(e) => {
                                    if e.kind() != std::io::ErrorKind::WouldBlock {
                                        logger_clone.log(&format!("[{}] TCP read error: {}", config_clone.id, e), 4);
                                        let mut clients = tcp_clients_clone.lock().unwrap();
                                        clients[0] = None;
                                        break;
                                    }
                                }
                            }
                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                    Err(e) => {
                        logger.log(&format!("[{}] Connection failed: {} - retrying in {:?}", 
                            config.id, e, reconnect_delay), 4);
                        thread::sleep(reconnect_delay);
                        reconnect_delay = (reconnect_delay * 2).min(max_reconnect_delay);
                    }
                }
            }
        });
        
        Ok(())
    }

    fn start_tcp_listener(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bind_address = format!("{}:{}", self.config.tcp_address, self.config.tcp_port);
        let listener = TcpListener::bind(&bind_address)?;
        self.logger.log(&format!("[{}] TCP listener on {} (max {} clients)", 
            self.config.id, bind_address, self.max_clients), 5);
        
        let tcp_clients = self.tcp_clients.clone();
        let max_clients = self.max_clients;
        let serial = self.serial_port.clone();
        let serial_peer = self.serial_peer.clone();
        let config = self.config.clone();
        let logger = self.logger.clone();
        
        // Accept loop thread
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(client_stream) => {
                        let peer_addr = client_stream.peer_addr().ok();
                        let mut clients = tcp_clients.lock().unwrap();
                        
                        // Find empty slot
                        let slot = clients.iter_mut()
                            .enumerate()
                            .find(|(_, c)| c.is_none());
                        
                        if let Some((index, slot_ref)) = slot {
                            let info = TcpClientInfo {
                                stream: client_stream.try_clone().unwrap(),
                                connected_at: std::time::SystemTime::now(),
                            };
                            *slot_ref = Some(info);
                            drop(clients);  // Release lock before spawning thread
                            
                            logger.log(&format!("[{}] Client {} connected from {:?}", 
                                config.id, index, peer_addr), 6);
                            
                            // Spawn read thread for this client
                            let tcp_clients_clone = tcp_clients.clone();
                            let serial_clone = serial.clone();
                            let serial_peer_clone = serial_peer.clone();
                            let config_clone = config.clone();
                            let logger_clone = logger.clone();
                            
                            thread::spawn(move || {
                                let mut buffer = vec![0u8; 4096];
                                let mut client_stream = client_stream;
                                
                                loop {
                                    match client_stream.read(&mut buffer) {
                                        Ok(0) => {
                                            logger_clone.log(&format!("[{}] Client {} disconnected", 
                                                config_clone.id, index), 6);
                                            let mut clients = tcp_clients_clone.lock().unwrap();
                                            clients[index] = None;
                                            break;
                                        }
                                        Ok(n) => {
                                            let data = &buffer[..n];
                                            
                                            if config_clone.dump_frames {
                                                logger_clone.log(&format!("[{}] Client {}->Serial ({} bytes): {:02x?}", 
                                                    config_clone.id, index, n, data), 7);
                                            }
                                            
                                            // Apply channel remapping if needed (TCP -> Serial)
                                            let processed = if config_clone.kiss_chan >= 0 && config_clone.kiss_chan <= 15 {
                                                // Remap KISS channel 0 back to configured channel
                                                Self::remap_kiss_channel_in(data, config_clone.kiss_port)
                                            } else if config_clone.raw_copy {
                                                data.to_vec()
                                            } else if config_clone.phil_flag {
                                                process_phil_flag_tcp_to_serial(data)
                                            } else {
                                                data.to_vec()
                                            };
                                            
                                            // KISSCOPY: Send to other clients
                                            if config_clone.kiss_copy {
                                                Self::broadcast_to_other_clients(
                                                    index, &processed, &tcp_clients_clone, &config_clone, &logger_clone
                                                );
                                            }
                                            
                                            // Send to serial
                                            if let Some(ref peer) = serial_peer_clone {
                                                let translated = Self::translate_kiss_port_static(
                                                    &processed, &config_clone, false, config_clone.xkiss_mode);
                                                if let Ok(mut port) = peer.lock() {
                                                    let _ = port.write_all(&translated);
                                                }
                                            } else {
                                                if let Ok(mut port) = serial_clone.lock() {
                                                    let _ = port.write_all(&processed);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            if e.kind() != std::io::ErrorKind::WouldBlock {
                                                logger_clone.log(&format!("[{}] Client {} read error: {}", 
                                                    config_clone.id, index, e), 4);
                                                let mut clients = tcp_clients_clone.lock().unwrap();
                                                clients[index] = None;
                                                break;
                                            }
                                        }
                                    }
                                    thread::sleep(Duration::from_millis(10));
                                }
                            });
                        } else {
                            drop(clients);
                            logger.log(&format!("[{}] Connection refused from {:?} - {} clients already connected", 
                                config.id, peer_addr, max_clients), 4);
                        }
                    }
                    Err(e) => {
                        logger.log(&format!("[{}] Accept error: {}", config.id, e), 3);
                    }
                }
            }
        });
        
        // Serial read thread (broadcasts to all connected TCP clients)
        let serial = self.serial_port.clone();
        let _serial_peer = self.serial_peer.clone();
        let tcp_clients = self.tcp_clients.clone();
        let agw_clients = self.agw_clients.clone();
        let agw_enabled = self.agw_enabled;
        let xkiss_buffer = self.xkiss_rx_buffer.clone();
        let config = self.config.clone();
        let logger = self.logger.clone();
        let pcap = self.pcap_writer.clone();
        
        thread::spawn(move || {
            let mut buffer = vec![0u8; 4096];
            let mut frame_buffer = Vec::new();
            let mut in_frame = false;
            
            loop {
                if let Ok(mut port) = serial.lock() {
                    match port.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            let data = &buffer[..n];
                            
                            if config.raw_copy {
                                // Raw copy mode: send to all clients
                                let clients = tcp_clients.lock().unwrap();
                                for (i, client) in clients.iter().enumerate() {
                                    if let Some(info) = client {
                                        if let Ok(mut stream) = info.stream.try_clone() {
                                            if let Err(e) = stream.write_all(data) {
                                                logger.log(&format!("[{}] Client {} write error: {}", 
                                                    config.id, i, e), 4);
                                            }
                                        }
                                    }
                                }
                                continue;
                            }
                            
                            for &byte in data {
                                if byte == KISS_FEND {
                                    if in_frame && !frame_buffer.is_empty() {
                                        let processed = if config.phil_flag {
                                            process_frame_with_phil_flag(&frame_buffer)
                                        } else {
                                            frame_buffer.clone()
                                        };
                                        
                                        if config.dump_frames {
                                            logger.log(&format!("[{}] Serial->Out ({} bytes): {:02x?}", 
                                                config.id, processed.len(), processed), 7);
                                        }
                                        
                                        if config.parse_kiss {
                                            parse_kiss_frame(&processed, &logger, &config.id);
                                        }
                                        
                                        if let Some(ref pcap) = pcap {
                                            if processed.len() > 1 {
                                                let _ = pcap.write_packet(&processed[1..]);
                                            }
                                        }
                                        
                                        let mut full_frame = vec![KISS_FEND];
                                        full_frame.extend_from_slice(&processed);
                                        full_frame.push(KISS_FEND);
                                        
                                        // Check channel filtering and apply channel remapping
                                        let should_send = if config.kiss_chan == -1 {
                                            true  // All channels
                                        } else if processed.len() > 0 {
                                            let kiss_byte = processed[0];
                                            let channel = (kiss_byte >> 4) & 0x0F;
                                            channel as i32 == config.kiss_chan
                                        } else {
                                            false
                                        };
                                        
                                        if should_send {
                                            // Apply channel remapping if needed (Serial -> TCP)
                                            let final_frame = if config.kiss_chan >= 0 && config.kiss_chan <= 15 && processed.len() > 0 {
                                                // Remap channel to 0 for application
                                                let mut remapped = full_frame.clone();
                                                let kiss_byte = remapped[1];
                                                let cmd = kiss_byte & 0x0F;
                                                remapped[1] = (0 << 4) | cmd;  // Channel 0
                                                remapped
                                            } else {
                                                full_frame.clone()
                                            };
                                            
                                            // If XKISS polling is enabled, buffer the packet
                                            if config.xkiss_mode && config.xkiss_polling {
                                                if let Ok(mut buf) = xkiss_buffer.lock() {
                                                    match buf.push(final_frame.clone()) {
                                                        Ok(_) => {
                                                            if config.dump_frames {
                                                                logger.log(&format!("[{}] Buffered packet ({} bytes, buffer has {})", 
                                                                    config.id, final_frame.len(), buf.len()), 7);
                                                            }
                                                        }
                                                        Err(e) => {
                                                            logger.log(&format!("[{}] Buffer full, dropping packet: {}", 
                                                                config.id, e), 4);
                                                        }
                                                    }
                                                }
                                            } else {
                                                // Send immediately to all TCP clients
                                                Self::send_to_all_tcp_clients(&final_frame, &tcp_clients, &config, &logger);
                                                
                                                // Also send to AGW clients if enabled
                                                if agw_enabled {
                                                    send_to_agw_clients(&final_frame, &agw_clients, 
                                                        config.agw_port, false, &logger, &config.id);
                                                }
                                            }
                                        }
                                        
                                        frame_buffer.clear();
                                    }
                                    in_frame = !in_frame;
                                } else if in_frame {
                                    frame_buffer.push(byte);
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                        Err(e) => {
                            logger.log(&format!("[{}] Serial read error: {}", config.id, e), 3);
                        }
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
        
        // XKISS polling thread (if enabled)
        if self.config.xkiss_mode && self.config.xkiss_polling {
            let xkiss_buffer = self.xkiss_rx_buffer.clone();
            let tcp_clients = self.tcp_clients.clone();
            let config = self.config.clone();
            let logger = self.logger.clone();
            let poll_interval = Duration::from_millis(self.config.xkiss_poll_timer_ms);
            
            thread::spawn(move || {
                loop {
                    thread::sleep(poll_interval);
                    
                    let packets = {
                        let mut buf = xkiss_buffer.lock().unwrap();
                        buf.poll_flush()
                    };
                    
                    if !packets.is_empty() {
                        if config.dump_frames {
                            logger.log(&format!("[{}] Polling flush: {} packets", 
                                config.id, packets.len()), 7);
                        }
                        
                        for packet in packets {
                            Self::send_to_all_tcp_clients(&packet, &tcp_clients, &config, &logger);
                        }
                    }
                }
            });
        }
        
        Ok(())
    }
    
    // Helper function to send data to all connected TCP clients
    fn send_to_all_tcp_clients(
        data: &[u8],
        tcp_clients: &Arc<Mutex<Vec<Option<TcpClientInfo>>>>,
        config: &CrossConnect,
        logger: &Arc<Logger>
    ) {
        let clients = tcp_clients.lock().unwrap();
        for (i, client) in clients.iter().enumerate() {
            if let Some(info) = client {
                if let Ok(mut stream) = info.stream.try_clone() {
                    if let Err(e) = stream.write_all(data) {
                        logger.log(&format!("[{}] Client {} write error: {}", 
                            config.id, i, e), 4);
                    }
                }
            }
        }
    }
    
    // Helper function for KISSCOPY: broadcast to other clients
    fn broadcast_to_other_clients(
        source_index: usize,
        data: &[u8],
        tcp_clients: &Arc<Mutex<Vec<Option<TcpClientInfo>>>>,
        config: &CrossConnect,
        logger: &Arc<Logger>
    ) {
        let clients = tcp_clients.lock().unwrap();
        for (i, client) in clients.iter().enumerate() {
            if i == source_index {
                continue;  // Skip source client
            }
            
            if let Some(info) = client {
                if let Ok(mut stream) = info.stream.try_clone() {
                    if let Err(e) = stream.write_all(data) {
                        logger.log(&format!("[{}] KISSCOPY to client {} error: {}", 
                            config.id, i, e), 4);
                    }
                }
            }
        }
    }
    
    // Helper function to remap KISS channel from app (channel 0) to configured channel
    fn remap_kiss_channel_in(data: &[u8], target_channel: u8) -> Vec<u8> {
        if data.len() < 2 || data[0] != KISS_FEND {
            return data.to_vec();
        }
        
        let mut result = data.to_vec();
        let kiss_byte = data[1];
        let cmd = kiss_byte & 0x0F;
        result[1] = (target_channel << 4) | cmd;
        result
    }


    fn start_agw_listener(&self, agw_address: String, agw_port: u16) 
        -> Result<(), Box<dyn std::error::Error>> {
        
        if !self.agw_enabled {
            return Ok(());  // AGW not enabled for this bridge
        }
        
        let bind_address = format!("{}:{}", agw_address, agw_port);
        let listener = TcpListener::bind(&bind_address)?;
        self.logger.log(&format!("[{}] AGW listener on {} (max {} clients)", 
            self.config.id, bind_address, self.max_clients), 5);
        
        let agw_clients = self.agw_clients.clone();
        let max_clients = self.max_clients;
        let serial = self.serial_port.clone();
        let config = self.config.clone();
        let logger = self.logger.clone();
        
        // Accept loop thread
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(client_stream) => {
                        let peer_addr = client_stream.peer_addr().ok();
                        let mut clients = agw_clients.lock().unwrap();
                        
                        // Find empty slot
                        let slot = clients.iter_mut()
                            .enumerate()
                            .find(|(_, c)| c.is_none());
                        
                        if let Some((index, slot_ref)) = slot {
                            let info = AgwClientInfo {
                                stream: client_stream.try_clone().unwrap(),
                                connected_at: std::time::SystemTime::now(),
                                registered_call: None,
                                monitor_enabled: false,
                            };
                            *slot_ref = Some(info);
                            drop(clients);
                            
                            logger.log(&format!("[{}] AGW client {} connected from {:?}", 
                                config.id, index, peer_addr), 6);
                            
                            // Spawn read thread for this AGW client
                            let agw_clients_clone = agw_clients.clone();
                            let serial_clone = serial.clone();
                            let config_clone = config.clone();
                            let logger_clone = logger.clone();
                            
                            thread::spawn(move || {
                                let mut client_stream = client_stream;
                                let mut buffer = vec![0u8; 8192];
                                
                                loop {
                                    match client_stream.read(&mut buffer) {
                                        Ok(0) => {
                                            logger_clone.log(&format!("[{}] AGW client {} disconnected", 
                                                config_clone.id, index), 6);
                                            let mut clients = agw_clients_clone.lock().unwrap();
                                            clients[index] = None;
                                            break;
                                        }
                                        Ok(n) => {
                                            let data = &buffer[..n];
                                            
                                            // Parse AGW frame(s) - there may be multiple
                                            let mut offset = 0;
                                            while offset + AgwHeader::SIZE <= data.len() {
                                                // Parse header
                                                if let Some(header) = AgwHeader::from_bytes(&data[offset..]) {
                                                    let data_len = header.data_len as usize;
                                                    let frame_end = offset + AgwHeader::SIZE + data_len;
                                                    
                                                    if frame_end > data.len() {
                                                        // Incomplete frame, wait for more data
                                                        break;
                                                    }
                                                    
                                                    let frame_data = if data_len > 0 {
                                                        &data[offset + AgwHeader::SIZE..frame_end]
                                                    } else {
                                                        &[]
                                                    };
                                                    
                                                    if config_clone.dump_frames {
                                                        let port = header.port;  // Copy to avoid packed field reference
                                                        logger_clone.log(&format!(
                                                            "[{}] AGW client {} RX: kind={:02x} port={} len={}", 
                                                            config_clone.id, index, header.kind, 
                                                            port, data_len), 7);
                                                    }
                                                    
                                                    // Handle AGW command
                                                    match header.kind {
                                                        b'G' => {
                                                            // Port Information Request
                                                            if let Err(e) = handle_agw_port_info(
                                                                &mut client_stream, &header) {
                                                                logger_clone.log(&format!(
                                                                    "[{}] AGW client {} error: {}", 
                                                                    config_clone.id, index, e), 4);
                                                            }
                                                        }
                                                        b'g' => {
                                                            // Port Capabilities Request
                                                            if let Err(e) = handle_agw_capabilities(
                                                                &mut client_stream, &header) {
                                                                logger_clone.log(&format!(
                                                                    "[{}] AGW client {} error: {}", 
                                                                    config_clone.id, index, e), 4);
                                                            }
                                                        }
                                                        b'X' => {
                                                            // Callsign Registration
                                                            if let Err(e) = handle_agw_register(
                                                                &mut client_stream, &header,
                                                                &agw_clients_clone, index) {
                                                                logger_clone.log(&format!(
                                                                    "[{}] AGW client {} error: {}", 
                                                                    config_clone.id, index, e), 4);
                                                            }
                                                        }
                                                        b'x' => {
                                                            // Callsign Unregister
                                                            let mut clients = agw_clients_clone.lock().unwrap();
                                                            if let Some(Some(ref mut client)) = clients.get_mut(index) {
                                                                client.registered_call = None;
                                                            }
                                                        }
                                                        b'M' => {
                                                            // Monitor Enable
                                                            handle_agw_monitor_enable(&agw_clients_clone, index);
                                                            logger_clone.log(&format!(
                                                                "[{}] AGW client {} monitor enabled", 
                                                                config_clone.id, index), 6);
                                                        }
                                                        b'm' => {
                                                            // Monitor Disable
                                                            handle_agw_monitor_disable(&agw_clients_clone, index);
                                                            logger_clone.log(&format!(
                                                                "[{}] AGW client {} monitor disabled", 
                                                                config_clone.id, index), 6);
                                                        }
                                                        b'K' => {
                                                            // Raw Data (transmit)
                                                            if config_clone.dump_frames {
                                                                logger_clone.log(&format!(
                                                                    "[{}] AGW client {} TX ({} bytes)", 
                                                                    config_clone.id, index, data_len), 7);
                                                            }
                                                            
                                                            // Convert AGW to KISS and send to serial
                                                            let kiss_frame = agw_to_kiss(&header, frame_data, &config_clone);
                                                            
                                                            if let Ok(mut port) = serial_clone.lock() {
                                                                if let Err(e) = port.write_all(&kiss_frame) {
                                                                    logger_clone.log(&format!(
                                                                        "[{}] Serial write error: {}", 
                                                                        config_clone.id, e), 4);
                                                                }
                                                            }
                                                        }
                                                        _ => {
                                                            // Unknown/unimplemented command
                                                            if config_clone.dump_frames {
                                                                logger_clone.log(&format!(
                                                                    "[{}] AGW client {} unknown command: {:02x}", 
                                                                    config_clone.id, index, header.kind), 7);
                                                            }
                                                        }
                                                    }
                                                    
                                                    offset = frame_end;
                                                } else {
                                                    // Invalid header
                                                    break;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            if e.kind() != std::io::ErrorKind::WouldBlock {
                                                logger_clone.log(&format!(
                                                    "[{}] AGW client {} read error: {}", 
                                                    config_clone.id, index, e), 4);
                                                let mut clients = agw_clients_clone.lock().unwrap();
                                                clients[index] = None;
                                                break;
                                            }
                                        }
                                    }
                                    thread::sleep(Duration::from_millis(10));
                                }
                            });
                        } else {
                            drop(clients);
                            logger.log(&format!(
                                "[{}] AGW connection refused from {:?} - {} clients already connected", 
                                config.id, peer_addr, max_clients), 4);
                        }
                    }
                    Err(e) => {
                        logger.log(&format!("[{}] AGW accept error: {}", config.id, e), 3);
                    }
                }
            }
        });
        
        Ok(())
    }
    fn translate_kiss_port_static(data: &[u8], config: &CrossConnect, from_xkiss: bool, to_xkiss: bool) -> Vec<u8> {
        if data.is_empty() || (!from_xkiss && !to_xkiss) {
            return data.to_vec();
        }

        let mut result = Vec::with_capacity(data.len());
        
        if from_xkiss && !to_xkiss {
            if let Some(_xkiss_port) = config.xkiss_port {
                for &byte in data {
                    if byte == KISS_FEND {
                        result.push(byte);
                    } else if result.len() == 1 {
                        let cmd_type = byte & 0x0F;
                        let new_byte = (config.kiss_port << 4) | cmd_type;
                        result.push(new_byte);
                    } else {
                        result.push(byte);
                    }
                }
            } else {
                result = data.to_vec();
            }
        } else if !from_xkiss && to_xkiss {
            if let Some(xkiss_port) = config.xkiss_port {
                for &byte in data {
                    if byte == KISS_FEND {
                        result.push(byte);
                    } else if result.len() == 1 {
                        let cmd_type = byte & 0x0F;
                        let new_byte = (xkiss_port << 4) | cmd_type;
                        result.push(new_byte);
                    } else {
                        result.push(byte);
                    }
                }
            } else {
                result = data.to_vec();
            }
        } else {
            result = data.to_vec();
        }
        
        result
    }
}

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

#[allow(dead_code)] // Used when xkiss_checksum is enabled
fn calculate_xkiss_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

#[allow(dead_code)] // Used when xkiss_checksum is enabled
fn verify_xkiss_checksum(frame: &[u8]) -> bool {
    if frame.len() < 2 {
        return false;
    }
    let data = &frame[..frame.len()-1];
    let checksum = frame[frame.len()-1];
    calculate_xkiss_checksum(data) == checksum
}

fn is_kiss_packet(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }
    data[0] == KISS_FEND && data[data.len()-1] == KISS_FEND
}

#[allow(dead_code)] // Used when reframe_large_packets is enabled
fn estimate_philflag_size(data: &[u8]) -> usize {
    let c0_count = data.iter().filter(|&&b| b == 0xC0).count();
    data.len() + c0_count
}

#[allow(dead_code)] // Used when reframe_large_packets is enabled
fn reframe_large_packet(packet: &[u8], phil_flag: bool, max_size: usize) -> Vec<Vec<u8>> {
    if packet.len() < 3 {
        return vec![packet.to_vec()];
    }
    
    let estimated_size = if phil_flag {
        estimate_philflag_size(packet)
    } else {
        packet.len()
    };
    
    if estimated_size <= max_size {
        return vec![packet.to_vec()];
    }
    
    let kiss_cmd = packet[1];
    let ax25_start = 2;
    let ax25_end = packet.len() - 1;
    
    if ax25_end <= ax25_start {
        return vec![packet.to_vec()];
    }
    
    let ax25_data = &packet[ax25_start..ax25_end];
    
    if ax25_data.len() < 14 {
        return vec![packet.to_vec()];
    }
    
    let mut info_start = 14;
    while info_start < ax25_data.len() && (ax25_data[info_start - 1] & 0x01) == 0 {
        info_start += 7;
        if info_start >= ax25_data.len() {
            return vec![packet.to_vec()];
        }
    }
    
    if info_start + 2 >= ax25_data.len() {
        return vec![packet.to_vec()];
    }
    
    info_start += 2;
    
    let header = &ax25_data[..info_start];
    let info = &ax25_data[info_start..];
    
    let conservative_chunk_size = (max_size / 2).min(220);
    let mut fragments = Vec::new();
    
    for chunk in info.chunks(conservative_chunk_size) {
        let mut frag = vec![KISS_FEND, kiss_cmd];
        frag.extend_from_slice(header);
        frag.extend_from_slice(chunk);
        frag.push(KISS_FEND);
        fragments.push(frag);
    }
    
    if fragments.is_empty() {
        vec![packet.to_vec()]
    } else {
        fragments
    }
}

// ============================================================================
// AGW (AGWPE) Protocol Functions
// ============================================================================

// Extract AX.25 addresses from packet (simplified - extracts FROM and TO calls)
fn extract_ax25_addresses(ax25_data: &[u8]) -> (String, String) {
    if ax25_data.len() < 14 {
        return (String::new(), String::new());
    }
    
    // Destination callsign (bytes 0-6)
    let to_call = extract_callsign(&ax25_data[0..7]);
    
    // Source callsign (bytes 7-13)
    let from_call = extract_callsign(&ax25_data[7..14]);
    
    (from_call, to_call)
}

fn extract_callsign(data: &[u8]) -> String {
    if data.len() < 7 {
        return String::new();
    }
    
    let mut call = String::new();
    
    // Extract callsign (first 6 bytes, shifted right by 1)
    for i in 0..6 {
        let c = (data[i] >> 1) & 0x7F;
        if c != 0x20 {  // Skip spaces
            call.push(c as char);
        }
    }
    
    // Extract SSID (7th byte)
    let ssid = (data[6] >> 1) & 0x0F;
    if ssid > 0 {
        call.push_str(&format!("-{}", ssid));
    }
    
    call
}

// Build AGW frame
fn build_agw_frame(kind: u8, port: u32, from: &str, to: &str, pid: u8, data: &[u8]) -> Vec<u8> {
    let mut header = AgwHeader::new();
    header.port = port;
    header.kind = kind;
    header.pid = pid;
    header.data_len = data.len() as u32;
    
    // Copy callsigns (truncate/pad to 10 bytes)
    let from_bytes = from.as_bytes();
    let to_bytes = to.as_bytes();
    
    for (i, &b) in from_bytes.iter().take(10).enumerate() {
        header.call_from[i] = b;
    }
    
    for (i, &b) in to_bytes.iter().take(10).enumerate() {
        header.call_to[i] = b;
    }
    
    let mut frame = header.to_bytes();
    frame.extend_from_slice(data);
    frame
}

// Convert KISS frame to AGW frame
fn kiss_to_agw(kiss_frame: &[u8], agw_port: u8, monitor_mode: bool) -> Vec<u8> {
    // Extract AX.25 data from KISS frame
    // KISS format: [FEND][CMD][AX.25 DATA][FEND]
    if kiss_frame.len() < 4 {
        return Vec::new();
    }
    
    // Skip FEND and CMD byte, take until final FEND
    let ax25_start = 2;
    let ax25_end = kiss_frame.len() - 1;
    
    if ax25_end <= ax25_start {
        return Vec::new();
    }
    
    let ax25_data = &kiss_frame[ax25_start..ax25_end];
    
    // Extract addresses
    let (from_call, to_call) = extract_ax25_addresses(ax25_data);
    
    // Determine kind: 'K' for normal data, 'U' for monitored
    let kind = if monitor_mode { b'U' } else { b'K' };
    
    // Build AGW frame
    build_agw_frame(kind, agw_port as u32, &from_call, &to_call, 0, ax25_data)
}

// Convert AGW frame to KISS frame with PhilFlag support
fn agw_to_kiss(_agw_header: &AgwHeader, agw_data: &[u8], config: &CrossConnect) -> Vec<u8> {
    // Extract AX.25 data from AGW frame
    let ax25_data = agw_data;
    
    // Apply PhilFlag escaping if enabled (same as KISS TCP clients)
    let processed_data = if config.phil_flag {
        process_phil_flag_tcp_to_serial(ax25_data)
    } else {
        ax25_data.to_vec()
    };
    
    // Build KISS frame
    let mut kiss_frame = vec![KISS_FEND];
    kiss_frame.push((config.kiss_port << 4) | 0x00);  // Data frame command
    kiss_frame.extend_from_slice(&processed_data);
    kiss_frame.push(KISS_FEND);
    
    kiss_frame
}

// Send AGW frame to client
fn send_agw_frame(stream: &mut TcpStream, frame: &[u8]) -> std::io::Result<()> {
    stream.write_all(frame)
}

// Handle AGW Port Information Request ('G')
fn handle_agw_port_info(stream: &mut TcpStream, header: &AgwHeader) -> std::io::Result<()> {
    let version_info = format!("rax25kb v1.7.3 AGW");
    let response = build_agw_frame(b'G', header.port, "", "", 0, version_info.as_bytes());
    send_agw_frame(stream, &response)
}

// Handle AGW Port Capabilities Request ('g')
fn handle_agw_capabilities(stream: &mut TcpStream, header: &AgwHeader) -> std::io::Result<()> {
    // Capabilities byte array (12 bytes)
    // Byte 0: Can monitor
    // Byte 1: Can transmit
    // Byte 2: Can hear
    // Other bytes: reserved
    let capabilities = [
        1,  // Can monitor
        1,  // Can transmit
        1,  // Can hear
        0, 0, 0, 0, 0, 0, 0, 0, 0  // Reserved
    ];
    
    let response = build_agw_frame(b'g', header.port, "", "", 0, &capabilities);
    send_agw_frame(stream, &response)
}

// Handle AGW Callsign Registration ('X')
fn handle_agw_register(stream: &mut TcpStream, header: &AgwHeader, 
                       agw_clients: &Arc<Mutex<Vec<Option<AgwClientInfo>>>>,
                       client_index: usize) -> std::io::Result<()> {
    // Extract callsign from call_from field
    let callsign = std::str::from_utf8(&header.call_from)
        .unwrap_or("")
        .trim_end_matches('\0')
        .to_string();
    
    // Store callsign in client info
    if let Ok(mut clients) = agw_clients.lock() {
        if let Some(Some(ref mut client)) = clients.get_mut(client_index) {
            client.registered_call = Some(callsign);
        }
    }
    
    // Send confirmation
    let response = build_agw_frame(b'X', header.port, "", "", 0, &[]);
    send_agw_frame(stream, &response)
}

// Handle AGW Monitor Enable ('M')
fn handle_agw_monitor_enable(agw_clients: &Arc<Mutex<Vec<Option<AgwClientInfo>>>>,
                             client_index: usize) {
    if let Ok(mut clients) = agw_clients.lock() {
        if let Some(Some(ref mut client)) = clients.get_mut(client_index) {
            client.monitor_enabled = true;
        }
    }
}

// Handle AGW Monitor Disable ('m')
fn handle_agw_monitor_disable(agw_clients: &Arc<Mutex<Vec<Option<AgwClientInfo>>>>,
                              client_index: usize) {
    if let Ok(mut clients) = agw_clients.lock() {
        if let Some(Some(ref mut client)) = clients.get_mut(client_index) {
            client.monitor_enabled = false;
        }
    }
}

// Send packet to all AGW clients (optionally only monitor-enabled ones)
fn send_to_agw_clients(kiss_frame: &[u8], 
                       agw_clients: &Arc<Mutex<Vec<Option<AgwClientInfo>>>>,
                       agw_port: u8,
                       monitor_only: bool,
                       logger: &Arc<Logger>,
                       config_id: &str) {
    let agw_frame = kiss_to_agw(kiss_frame, agw_port, monitor_only);
    
    if agw_frame.is_empty() {
        return;
    }
    
    let clients = agw_clients.lock().unwrap();
    for (i, client_opt) in clients.iter().enumerate() {
        if let Some(client) = client_opt {
            // If monitor_only is true, only send to clients with monitor enabled
            if monitor_only && !client.monitor_enabled {
                continue;
            }
            
            if let Ok(mut stream) = client.stream.try_clone() {
                if let Err(e) = stream.write_all(&agw_frame) {
                    logger.log(&format!("[{}] AGW client {} write error: {}", 
                        config_id, i, e), 4);
                }
            }
        }
    }
}

fn write_pidfile(pidfile: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(pidfile)?;
    writeln!(file, "{}", std::process::id())?;
    Ok(())
}

fn show_help(program_name: &str) {
    println!("rax25kb - AX.25 KISS Bridge v1.7.3\n");
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
    println!("  -I, --address <addr>  TCP address");
    println!("  -p, --port <port>     TCP port (default: 8001)");
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
    println!("Config file supports multiple cross-connect objects (cross_connect0000 to cross_connect9999)");
    println!("Each cross-connect can link serial ports to TCP or other serial ports with KISS/XKISS translation\n");
    println!("For more information, see: https://github.com/ke4ahr/rax25kb/");
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
        println!("rax25kb - AX.25 KISS Bridge v1.7.3");
        println!("====================================");
        println!("Configuration from: {}", config_file);
        println!("Cross-connects: {}", config.cross_connects.len());
        for cc in &config.cross_connects {
            println!("\n  [{}]", cc.id);
            println!("    Serial: {} @ {} baud", cc.serial_port, cc.baud_rate);
            if let Some(ref peer) = cc.serial_to_serial {
                println!("    -> Serial peer: {}", peer);
            } else {
                println!("    -> TCP: {}:{}", cc.tcp_address, cc.tcp_port);
            }
            if cc.xkiss_mode {
                println!("    XKISS mode: port {}", cc.xkiss_port.unwrap_or(0));
            } else {
                println!("    KISS port: {}", cc.kiss_port);
            }
            println!("    PhilFlag: {}", if cc.phil_flag { "ON" } else { "OFF" });
            println!("    Raw copy: {}", if cc.raw_copy { "ON" } else { "OFF" });
        }
        println!("\n  Log level: {}", config.log_level);
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
    logger.log("rax25kb v1.7.3 starting", 5);

    let pcap_writer = if let Some(ref pcap_path) = config.pcap_file {
        Some(Arc::new(PcapWriter::new(pcap_path)?))
    } else {
        None
    };

    let max_tcp_clients = config.max_tcp_clients;
    let mut bridges: Vec<CrossConnectBridge> = Vec::new();
    let mut serial_handles: HashMap<String, Arc<Mutex<Box<dyn SerialPort>>>> = HashMap::new();
    
    // First pass: Create primary ports (KISS port 0) and collect serial handles
    for cc in config.cross_connects.iter().filter(|cc| cc.is_primary_port) {
        logger.log(&format!("[{}] Creating primary port (KISS port {})", cc.id, cc.kiss_port), 6);
        let bridge = CrossConnectBridge::new(cc.clone(), max_tcp_clients, None, logger.clone(), pcap_writer.clone())?;
        serial_handles.insert(cc.serial_port.clone(), bridge.serial_port.clone());
        bridges.push(bridge);
    }
    
    // Second pass: Create secondary ports (KISS ports 1-15) with shared handles
    for cc in config.cross_connects.iter().filter(|cc| !cc.is_primary_port) {
        logger.log(&format!("[{}] Creating secondary port (KISS port {}) sharing {}", 
            cc.id, cc.kiss_port, cc.serial_port), 6);
        let shared = serial_handles.get(&cc.serial_port).cloned();
        if shared.is_none() {
            logger.log(&format!("[{}] WARNING: No primary port found for serial {}, creating as primary", 
                cc.id, cc.serial_port), 4);
        }
        let bridge = CrossConnectBridge::new(cc.clone(), max_tcp_clients, shared, logger.clone(), pcap_writer.clone())?;
        bridges.push(bridge);
    }
    
    for bridge in &bridges {
        if bridge.config.serial_to_serial.is_none() {
            match bridge.config.tcp_mode {
                TcpMode::Server => {
                    bridge.start_tcp_listener()?;
                }
                TcpMode::Client => {
                    if bridge.config.tcp_to_tcp_dangerous {
                        logger.log(&format!("[{}] WARNING: TCP-to-TCP mode enabled - use with caution!", 
                            bridge.config.id), 4);
                    }
                    bridge.start_tcp_client()?;
                }
                TcpMode::None => {
                    logger.log(&format!("[{}] No TCP mode configured, serial only", 
                        bridge.config.id), 6);
                }
            }
        }
    }
    
    // Start AGW server if enabled globally
    if config.agw_server_enable {
        logger.log(&format!("Starting AGW server on {}:{} (max {} clients)", 
            config.agw_server_address, config.agw_server_port, config.agw_max_clients), 5);
        
        // Start AGW listener on each bridge with AGW enabled
        for bridge in &bridges {
            if bridge.agw_enabled {
                bridge.start_agw_listener(
                    config.agw_server_address.clone(),
                    config.agw_server_port
                )?;
                logger.log(&format!("[{}] AGW enabled on port {}", 
                    bridge.config.id, bridge.config.agw_port), 6);
            }
        }
    }
    
    logger.log("All cross-connects started", 5);
    
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn parse_kiss_frame(frame: &[u8], logger: &Logger, bridge_id: &str) {
    if frame.is_empty() {
        return;
    }
    
    let port = (frame[0] >> 4) & 0x0F;
    let command = frame[0] & 0x0F;
    
    let cmd_str = match command {
        0 => "Data Frame",
        1 => "TX Delay",
        2 => "P Persistence",
        3 => "Slot Time",
        4 => "TX Tail",
        5 => "Full Duplex",
        6 => "Set Hardware",
        15 => "Return",
        _ => "Unknown",
    };
    
    logger.log(&format!("[{}] KISS: Port={}, Cmd={} ({}), Len={}", 
                       bridge_id, port, command, cmd_str, frame.len()), 6);
    
    if command == 0 && frame.len() > 1 {
        let ax25_data = &frame[1..];
        if ax25_data.len() >= 14 {
            let dest_callsign = extract_callsign(&ax25_data[0..7]);
            let src_callsign = extract_callsign(&ax25_data[7..14]);
            logger.log(&format!("[{}] AX.25: {} -> {}", bridge_id, src_callsign, dest_callsign), 6);
        }
    }
}

