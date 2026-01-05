# rax25kb Architecture

## Overview

rax25kb is a multi-port AX.25 KISS bridge that provides flexible connectivity between serial KISS/XKISS TNCs and TCP networks, as well as serial-to-serial cross-connections with protocol translation.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         rax25kb                             │
│                                                             │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
│  │ Cross-Connect  │  │ Cross-Connect  │  │ Cross-Connect  │ │
│  │     0000       │  │     0001       │  │     0002       │ │
│  │                │  │                │  │                │ │
│  │ ┌──────────┐   │  │ ┌──────────┐   │  │ ┌──────────┐   │ │
│  │ │ Serial   │   │  │ │ Serial   │   │  │ │ Serial   │   │ │
│  │ │ Port     │───┼──┼─│ Port     │   │  │ │ Port     │   │ │
│  │ └──────────┘   │  │ └──────────┘   │  │ └──────────┘   │ │
│  │      │         │  │      │         │  │      │         │ │
│  │      │         │  │      │         │  │      │         │ │
│  │      ▼         │  │      ▼         │  │      ▼         │ │
│  │ ┌──────────┐   │  │ ┌──────────┐   │  │ ┌──────────┐   │ │
│  │ │ TCP      │   │  │ │ Serial   │   │  │ │ TCP      │   │ │
│  │ │ Listener │   │  │ │ Peer     │   │  │ │ Listener │   │ │
│  │ └──────────┘   │  │ └──────────┘   │  │ └──────────┘   │ │
│  └────────────────┘  └────────────────┘  └────────────────┘ │ 
│                                                             │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Logger & PCAP Writer                      │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Configuration System

The configuration system supports two modes:

#### Legacy Mode (Backward Compatible)
- Single serial port to TCP bridge
- Defined by traditional config parameters:
  - `serial_port`
  - `tcp_address` 
  - `tcp_port`
  - Individual feature flags

#### Cross-Connect Mode (New in 1.6.5)
- Multiple independent bridges
- Each defined by `cross_connectXXXX` prefix (0000-9999)
- Per-bridge configuration:
  - Serial port settings (device, baud, flow control, parity, stop bits)
  - Destination (TCP or serial peer)
  - KISS/XKISS translation parameters
  - Feature flags (PhilFlag, dump, parse, raw copy)

### 2. CrossConnectBridge

Each `CrossConnectBridge` instance represents an independent connection path:

```rust
struct CrossConnectBridge {
    config: CrossConnect,              // Bridge configuration
    serial_port: Arc<Mutex<...>>,      // Source serial port
    tcp_client: Arc<Mutex<...>>,       // TCP client (if TCP mode)
    serial_peer: Option<Arc<...>>,     // Peer serial port (if serial mode)
    logger: Arc<Logger>,               // Logging system
    pcap_writer: Option<Arc<...>>,     // Packet capture
}
```

#### Bridge Modes

1. **Serial-to-TCP Mode**
   - Listens on specified TCP address:port
   - Accepts single client connection
   - Bridges serial ↔ TCP bidirectionally
   - Applies KISS processing and PhilFlag correction

2. **Serial-to-Serial Mode**
   - Direct connection between two serial ports
   - Supports KISS ↔ XKISS translation
   - Port number translation between different TNC ports
   - No TCP involvement

### 3. Data Flow

#### Serial → Destination

```
Serial Port Read
      ↓
  Frame Detection (KISS_FEND delimiters)
      ↓
  Frame Buffer Assembly
      ↓
  PhilFlag Processing (if enabled)
      ↓
  KISS Parsing (if enabled)
      ↓
  PCAP Logging (if enabled)
      ↓
  Protocol Translation (if needed)
      ↓
  Write to Destination (TCP or Serial)
```

#### Destination → Serial

```
Source Read (TCP or Serial)
      ↓
  PhilFlag Processing (if enabled)
      ↓
  Protocol Translation (if needed)
      ↓
  Write to Serial Port
```

### 4. KISS/XKISS Translation

The system supports bidirectional translation between standard KISS and Extended KISS:

- **Standard KISS**: Port number in upper nibble of command byte
  - Format: `[FEND][Port<<4 | Command][Data][FEND]`
  - Port range: 0-15

- **Extended KISS (XKISS)**: Separate port addressing
  - Allows more flexible port mapping
  - Configured per cross-connect

Translation occurs at the frame level, modifying the command byte to reflect the target port addressing scheme.

### 5. PhilFlag Processing

Addresses specific TASCO modem chipset bugs:

#### Serial → TCP/Serial
- Escapes embedded `0xC0` bytes within frame
- Converts: `0xC0` → `0xDB 0xDC` (FESC TFEND sequence)
- Prevents: Premature frame termination

#### TCP/Serial → Serial  
- Escapes 'C' characters to prevent `TC0\n` parsing
- Converts: `0x43` (C) → `0xDB 0x43`
- Converts: `0x63` (c) → `0xDB 0x63`

### 6. Threading Model

Each `CrossConnectBridge` spawns multiple threads:

1. **TCP Accept Thread** (TCP mode only)
   - Listens for incoming connections
   - Enforces single-connection limit
   - Spawns client handler threads

2. **TCP → Serial Thread** (per client)
   - Reads from TCP socket
   - Processes and writes to serial

3. **Serial → Destination Thread**
   - Reads from serial port
   - Assembles KISS frames
   - Distributes to destinations

All threads use `Arc<Mutex<>>` for safe shared access to ports and clients.

### 7. Logging System

Unified logging across all bridges:

- **Levels**: 0-9 (EMERG to VERBOSE)
- **Outputs**: Console, file, or both
- **Format**: `[timestamp] [level] [bridge_id] message`
- **Thread-safe**: Uses `Arc<Mutex<File>>`

### 8. PCAP Capture

Optional packet capture to standard PCAP format:

- **Protocol**: AX.25 (linktype 3)
- **Scope**: Can be enabled per-bridge or globally
- **Format**: Standard libpcap format
- **Content**: Raw AX.25 frames (KISS header stripped)

## Configuration File Format

### Legacy Format
```ini
serial_port=/dev/ttyUSB0
baud_rate=9600
tcp_address=0.0.0.0
tcp_port=8001
phil_flag=yes
```

### Cross-Connect Format
```ini
# First bridge
cross_connect0000.serial_port=/dev/ttyUSB0
cross_connect0000.baud_rate=9600
cross_connect0000.tcp_address=0.0.0.0
cross_connect0000.tcp_port=8001
cross_connect0000.phil_flag=yes

# Second bridge with XKISS translation
cross_connect0001.serial_port=/dev/ttyUSB1
cross_connect0001.baud_rate=9600
cross_connect0001.kiss_port=0
cross_connect0001.xkiss_mode=yes
cross_connect0001.xkiss_port=5
cross_connect0001.serial_to_serial=/dev/ttyUSB0

# Third bridge
cross_connect0002.serial_port=/dev/ttyUSB2
cross_connect0002.tcp_address=192.168.1.100
cross_connect0002.tcp_port=8002
```

## Extension Points

### Adding New Features

1. **Per-Bridge Features**: Add to `CrossConnect` struct
2. **Global Features**: Add to `Config` struct
3. **Processing Pipeline**: Modify frame processing functions
4. **Protocol Translation**: Extend translation functions

### Supporting New Protocols

The architecture can be extended to support protocols beyond KISS/XKISS by:

1. Adding protocol-specific structs
2. Implementing translation functions
3. Modifying frame detection logic
4. Adding configuration parameters

## Performance Considerations

- **Threading**: Each bridge operates independently
- **Buffer Sizes**: 4KB buffers for serial and TCP
- **Polling**: 10ms sleep between read attempts
- **Locking**: Minimal lock contention via separate mutexes
- **Memory**: Frame assembly uses dynamic vectors

## Security Considerations

- **TCP Binding**: Can bind to specific interfaces
- **Single Connection**: Prevents connection flooding
- **No Authentication**: Raw TCP, not encrypted
- **Local Use**: Designed for trusted network environments

## Future Architecture Enhancements

Potential areas for expansion:

1. **TLS/SSL Support**: Encrypted TCP connections
2. **Authentication**: Client authentication mechanisms
3. **Web Interface**: Configuration and monitoring GUI
4. **Dynamic Configuration**: Runtime bridge management
5. **Statistics**: Performance metrics and counters
6. **Plugin System**: User-defined processing modules

## Version History

- **1.6.5**: Multi-port cross-connect architecture
- **1.6.4**: Single-port serial-to-TCP bridge (legacy)

Copyright (C) 2025-2026 Kris Kirby, KE4AHR
Released under GPLv3.0.
