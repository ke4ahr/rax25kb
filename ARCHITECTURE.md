# rax25kb Architecture

## Table of Contents

- [Overview](#overview)
- [System Architecture](#system-architecture)
- [Core Components](#core-components)
- [Data Flow](#data-flow)
- [Threading Model](#threading-model)
- [Protocol Handling](#protocol-handling)
- [State Management](#state-management)
- [Error Handling](#error-handling)
- [Performance Considerations](#performance-considerations)
- [Security Considerations](#security-considerations)
- [Platform-Specific Implementation](#platform-specific-implementation)
- [Extension Points](#extension-points)

## Overview

rax25kb is a high-performance bridge application written in Rust that connects serial port KISS TNCs (Terminal Node Controllers) to TCP/IP networks. The architecture is designed for reliability, performance, and cross-platform compatibility.

### Design Goals

- **Simplicity**: Minimal complexity, focused functionality
- **Performance**: Low latency, high throughput
- **Reliability**: Robust error handling and recovery
- **Portability**: Cross-platform support (Linux, Windows, macOS)
- **Maintainability**: Clean code structure, well-documented
- **Zero Dependencies on Runtime**: No external libraries required at runtime

### Key Characteristics

- **Single-threaded per connection**: Each TCP connection spawns two threads (read/write)
- **Blocking I/O**: Simple, predictable behavior
- **No async runtime**: Reduces complexity and dependencies
- **Direct hardware access**: No middleware layers
- **Stateless operation**: No persistent state between connections

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         rax25kb                              │
│                                                              │
│  ┌────────────┐     ┌──────────────┐     ┌──────────────┐  │
│  │   Config   │────▶│ KissBridge   │────▶│   Logger     │  │
│  │   Parser   │     │              │     │              │  │
│  └────────────┘     └──────┬───────┘     └──────────────┘  │
│                            │                                │
│                            │                                │
│              ┌─────────────┴─────────────┐                  │
│              │                           │                  │
│              ▼                           ▼                  │
│     ┌────────────────┐         ┌─────────────────┐         │
│     │ Serial Manager │         │  TCP Listener   │         │
│     │                │         │                 │         │
│     └────────┬───────┘         └────────┬────────┘         │
│              │                          │                  │
└──────────────┼──────────────────────────┼──────────────────┘
               │                          │
               ▼                          ▼
        ┌─────────────┐           ┌─────────────┐
        │   Serial    │           │     TCP     │
        │   Port      │           │   Clients   │
        │  (TNC/      │           │   (APRS     │
        │   Modem)    │           │  Software)  │
        └─────────────┘           └─────────────┘
```

### Component Hierarchy

```
main()
├── Config::from_file()
│   └── apply_cli_overrides()
├── Logger::new()
├── KissBridge::new()
│   ├── SerialPort::open()
│   └── PcapWriter::new() [optional]
└── KissBridge::start_server()
    ├── TcpListener::bind() [multiple]
    └── loop: accept connections
        └── handle_client()
            ├── thread: serial_to_tcp
            └── loop: tcp_to_serial
```

## Core Components

### 1. Configuration System

**File**: Configuration parsing and management

**Responsibilities**:
- Parse configuration files (key=value format)
- Process command-line arguments
- Validate settings
- Apply overrides (CLI over file)
- Provide defaults

**Key Structures**:
```rust
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
    pcap_file: Option<String>,
    raw_copy: bool,
    // ... other flags
}
```

**Design Decisions**:
- Simple key=value format for ease of use
- Command-line overrides for flexibility
- Sensible defaults for common use cases
- Type-safe enums for options (FlowControl, StopBits, Parity)

### 2. Serial Port Manager

**Responsibilities**:
- Open and configure serial port
- Set baud rate, parity, stop bits, flow control
- Handle platform-specific serial port access
- Read/write raw bytes
- Manage timeouts

**Implementation**:
```rust
struct KissBridge {
    serial_port: Arc<Mutex<Box<dyn SerialPort>>>,
    config: Config,
    logger: Arc<Logger>,
    pcap_writer: Option<Arc<PcapWriter>>,
}
```

**Key Features**:
- Thread-safe access via `Arc<Mutex<>>`
- Dynamic dispatch for platform-specific implementations
- Non-blocking timeout-based I/O
- Configurable serial parameters

### 3. TCP Listener

**Responsibilities**:
- Bind to one or more TCP addresses
- Accept incoming connections
- Spawn handler threads for each connection
- Handle IPv4 and IPv6

**Implementation**:
- Multiple `TcpListener` instances for multi-address binding
- Non-blocking accept loop with polling
- Sequential connection handling (one at a time per listener)

### 4. Connection Handler

**Responsibilities**:
- Bridge data between serial and TCP
- Spawn bidirectional data transfer threads
- Apply PhilFlag corrections if enabled
- Parse and log KISS frames if enabled

**Architecture**:
```
TCP Client Connection
    │
    ├─▶ Thread 1: Serial → TCP (Read from serial, write to TCP)
    │   │
    │   ├─ Read from serial port (blocking with timeout)
    │   ├─ Apply PhilFlag corrections (if enabled)
    │   ├─ Parse KISS frames (if enabled)
    │   └─ Write to TCP socket
    │
    └─▶ Main Thread: TCP → Serial (Read from TCP, write to serial)
        │
        ├─ Read from TCP socket (blocking)
        ├─ Parse KISS frames (if enabled)
        ├─ Apply PhilFlag corrections (if enabled)
        └─ Write to serial port
```

### 5. KISS Protocol Handler

**Responsibilities**:
- Parse KISS frame structure
- Extract command and data
- Decode AX.25 headers
- Display frame information

**Key Structures**:
```rust
// KISS special bytes
const KISS_FEND: u8 = 0xC0;  // Frame delimiter
const KISS_FESC: u8 = 0xDB;  // Escape character
const KISS_TFEND: u8 = 0xDC; // Transposed FEND
const KISS_TFESC: u8 = 0xDD; // Transposed FESC
```

**Frame Parsing**:
```rust
fn parse_kiss_frame(&self, data: &[u8], direction: &str) {
    // 1. Find frame boundaries (FEND...FEND)
    // 2. Extract command byte (port + command)
    // 3. Extract data payload
    // 4. Parse AX.25 if data frame
    // 5. Display/log information
}
```

### 6. AX.25 Protocol Handler

**Responsibilities**:
- Parse AX.25 frame headers
- Extract callsigns and SSIDs
- Decode digipeater paths
- Identify frame types
- Display frame information

**Key Structures**:
```rust
struct AX25Address {
    callsign: String,  // Up to 6 characters
    ssid: u8,          // 0-15
}

struct AX25Frame {
    destination: AX25Address,
    source: AX25Address,
    digipeaters: Vec<AX25Address>,
    control: u8,
    pid: Option<u8>,
    info: Vec<u8>,
}

enum AX25FrameType {
    IFrame,   // Information (connected mode)
    SFrame,   // Supervisory (flow control)
    UFrame,   // Unnumbered (commands)
    UIFrame,  // Unnumbered Info (APRS)
    Unknown,
}
```

### 7. PhilFlag Processor

**Responsibilities**:
- Correct TASCO modem KISS escaping bugs
- Bidirectional processing (serial→TCP and TCP→serial)
- Frame boundary detection
- Escape sequence insertion

**Implementation**:

**Serial → TCP (Receive)**:
```rust
fn process_frame_with_phil_flag(frame: &[u8]) -> Vec<u8> {
    // Scan frame for unescaped 0xC0 bytes
    // Replace 0xC0 → 0xDB 0xDC
    // Preserve frame delimiters
}
```

**TCP → Serial (Transmit)**:
```rust
fn process_phil_flag_tcp_to_serial(data: &[u8]) -> Vec<u8> {
    // Scan for 'C' (0x43) and 'c' (0x63)
    // Escape as 0xDB 0x43 and 0xDB 0x63
    // Prevents "TC0\n" interpretation
}
```

### 8. Logger

**Responsibilities**:
- Multi-level logging (0-9)
- Console and/or file output
- Timestamp formatting
- Thread-safe logging

**Implementation**:
```rust
struct Logger {
    file: Option<Arc<Mutex<File>>>,
    log_level: u8,
    log_to_console: bool,
}
```

**Log Levels**:
- 0: EMERG (system unusable)
- 3: ERROR (error conditions)
- 5: NOTICE (normal significant, default)
- 7: DEBUG (debug messages)
- 9: VERBOSE (maximum verbosity)

### 9. PCAP Writer

**Responsibilities**:
- Write AX.25 frames to PCAP format
- Wireshark-compatible output
- Timestamp packets
- Thread-safe file access

**Format**:
- PCAP global header (24 bytes)
- Per-packet headers (16 bytes each)
- Data link type: 147 (USER0)
- Only captures KISS data frames (command 0)

## Data Flow

### Normal Operation Flow

```
1. Startup
   ├─ Parse configuration
   ├─ Open serial port
   ├─ Bind TCP listeners
   └─ Enter accept loop

2. Client Connection
   ├─ Accept TCP connection
   ├─ Clone serial port handle
   ├─ Spawn serial→TCP thread
   └─ Start TCP→serial loop

3. Serial → TCP Path
   ├─ Read from serial (with timeout)
   ├─ Buffer until FEND received (PhilFlag mode)
   ├─ Apply PhilFlag corrections (if enabled)
   ├─ Parse KISS frame (if enabled)
   ├─ Write to PCAP (if enabled)
   └─ Write to TCP socket

4. TCP → Serial Path
   ├─ Read from TCP socket
   ├─ Parse KISS frame (if enabled)
   ├─ Apply PhilFlag corrections (if enabled)
   └─ Write to serial port

5. Connection Close
   ├─ Detect disconnect
   ├─ Clean up threads
   ├─ Close TCP socket
   └─ Return to accept loop
```

### Raw Copy Mode Flow

```
Serial → TCP:
  Serial Port → Read → Write → TCP Socket

TCP → Serial:
  TCP Socket → Read → Write → Serial Port

No processing, parsing, or modifications
```

### KISS Frame Processing Flow

```
Incoming Data (Serial)
    ↓
Buffer Accumulation
    ↓
Frame Detection (FEND...FEND)
    ↓
┌────────────────────┐
│  PhilFlag Enabled? │
└─────────┬──────────┘
          ├─ Yes → Escape unescaped 0xC0
          └─ No  → Pass through
    ↓
┌────────────────────┐
│ Parse KISS Enabled?│
└─────────┬──────────┘
          ├─ Yes → Extract command, parse AX.25
          └─ No  → Skip parsing
    ↓
┌────────────────────┐
│   PCAP Enabled?    │
└─────────┬──────────┘
          ├─ Yes → Write to PCAP file
          └─ No  → Skip capture
    ↓
Forward to TCP
```

## Threading Model

### Thread Architecture

```
Main Thread
├─ Configuration & Initialization
├─ Signal Handler (Ctrl+C)
└─ Accept Loop
    └─ For each connection:
        ├─ Spawn Thread: serial_to_tcp
        │   └─ Loop: Read serial → Write TCP
        │
        └─ Main Thread: tcp_to_serial
            └─ Loop: Read TCP → Write serial
```

### Thread Safety

**Shared Resources**:
- Serial port: `Arc<Mutex<Box<dyn SerialPort>>>`
- Logger: `Arc<Logger>` (internal `Arc<Mutex<File>>`)
- PCAP writer: `Arc<PcapWriter>` (internal `Arc<Mutex<File>>`)

**Synchronization**:
- Mutex for serial port access (only one thread reads/writes at a time)
- Mutex for log file writes
- Mutex for PCAP file writes
- No other shared state between threads

### Thread Lifecycle

```
Connection Established
    ↓
Spawn serial_to_tcp thread
    ↓
Both threads running concurrently
    ├─ serial_to_tcp: Loops until error or disconnect
    └─ tcp_to_serial: Loops until error or disconnect
    ↓
Either thread exits
    ↓
Main thread drops serial_to_tcp thread
    ↓
Connection cleaned up
    ↓
Return to accept loop
```

## Protocol Handling

### KISS Protocol Implementation

**Frame Structure**:
```
┌──────┬──────────┬────────────────┬──────┐
│ FEND │ CMD_BYTE │      DATA      │ FEND │
└──────┴──────────┴────────────────┴──────┘
  0xC0   Port+Cmd   Escaped bytes   0xC0
```

**Command Byte Encoding**:
```
Bits 7-4: Port number (0-15)
Bits 3-0: Command (0-15)

Example: 0x00 = Port 0, Command 0 (Data Frame)
         0x10 = Port 1, Command 0 (Data Frame)
         0x01 = Port 0, Command 1 (TX Delay)
```

**Escape Sequences**:
```
Data Byte    →  KISS Encoding
─────────────────────────────
0xC0 (FEND)  →  0xDB 0xDC
0xDB (FESC)  →  0xDB 0xDD
```

### AX.25 Protocol Implementation

**Address Field Encoding**:
```
Each address: 7 bytes
  Bytes 0-5: Callsign (ASCII << 1)
  Byte 6:    SSID byte

SSID Byte:
  Bit 7:   C/R bit
  Bits 6-5: Reserved (11)
  Bits 4-1: SSID (0-15)
  Bit 0:   Address extension (0=more, 1=last)

Example: "KE4AHR-7"
  'K' << 1 = 0x96
  'E' << 1 = 0x8A
  '4' << 1 = 0x68
  'A' << 1 = 0x82
  'H' << 1 = 0x90
  'R' << 1 = 0xA4
  SSID byte = (7 << 1) | 0x61 = 0xEF
```

**Control Field**:
```
I-Frame:  N(R) P N(S) 0
S-Frame:  N(R) P/F S S 0 1
U-Frame:  M M M P/F M M 1 1
```

**Frame Type Detection**:
```rust
if (control & 0x01) == 0 → I-Frame
else if (control & 0x03) == 0x01 → S-Frame
else if (control & 0x03) == 0x03 → U-Frame or UI-Frame
```

## State Management

### Application State

rax25kb is largely **stateless** - it does not maintain connection state between sessions.

**Persistent State**:
- Configuration (read at startup)
- Log file (append-only)
- PCAP file (append-only)
- PID file (overwritten at startup)

**Runtime State**:
- Serial port handle (per connection)
- TCP socket (per connection)
- Frame buffers (per connection)
- Thread handles (per connection)

**No State Carried Between Connections**:
- No packet history
- No statistics tracking
- No client identification
- No authentication state

### Connection State

**Per-Connection State**:
```rust
struct ConnectionState {
    // Implicit in thread local storage
    frame_buffer: Vec<u8>,  // PhilFlag mode only
    in_frame: bool,         // PhilFlag mode only
}
```

**State Lifetime**:
- Created: Connection accepted
- Used: During data transfer
- Destroyed: Connection closed

## Error Handling

### Error Handling Strategy

**Principle**: Fail gracefully, log errors, continue operation when possible.

**Error Categories**:

1. **Fatal Errors** (exit program):
   - Cannot open serial port
   - Cannot bind to any TCP address
   - Invalid configuration
   - Missing required config options

2. **Connection Errors** (close connection, continue):
   - TCP read/write errors
   - Serial read/write errors
   - Client disconnect
   - Timeout exceeded

3. **Processing Errors** (log, continue):
   - Invalid KISS frames
   - Malformed AX.25 headers
   - PCAP write failures
   - Log write failures

### Error Recovery

**Serial Port Errors**:
```rust
match port.read(&mut buffer) {
    Ok(n) if n > 0 => { /* process data */ }
    Ok(_) => thread::sleep(Duration::from_millis(10)),
    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
        thread::sleep(Duration::from_millis(10));
        // Continue - timeout is normal
    }
    Err(e) => {
        logger.log(&format!("Error reading from serial: {}", e), 3);
        break; // Exit thread, close connection
    }
}
```

**TCP Errors**:
```rust
match stream.read(&mut buffer) {
    Ok(n) if n > 0 => { /* process data */ }
    Ok(_) => { 
        logger.log("Client disconnected", 5);
        break; // Normal disconnect
    }
    Err(e) => {
        logger.log(&format!("Error reading from TCP: {}", e), 3);
        break; // Exit loop, close connection
    }
}
```

**Parsing Errors**:
- Log error
- Skip malformed frame
- Continue processing

## Performance Considerations

### Optimization Strategies

1. **Minimal Copying**:
   - Direct buffer transfers where possible
   - Avoid unnecessary allocations
   - Reuse buffers in loops

2. **Efficient I/O**:
   - Blocking I/O (simpler than async)
   - Timeouts prevent indefinite blocking
   - Buffer sizes tuned for typical packet sizes (1024 bytes)

3. **Lock Minimization**:
   - Serial port mutex held only during I/O
   - Logger mutex held only during write
   - PCAP mutex held only during write

4. **Conditional Processing**:
   - PhilFlag only when enabled
   - KISS parsing only when enabled
   - Hex dump only when enabled
   - PCAP only when enabled

### Performance Characteristics

**Throughput**:
- Limited by serial baud rate
- TCP overhead negligible
- Processing overhead < 5% CPU

**Latency**:
- Serial read timeout: 100ms default
- Processing latency: < 1ms typical
- PhilFlag processing: < 1ms additional

**Memory Usage**:
- Base: ~5 MB
- Per connection: ~20 KB (buffers)
- PCAP file: grows with captured packets
- Log file: grows with log messages

**CPU Usage**:
- Idle: < 1%
- Normal operation: 1-5%
- With parsing/dump: 2-10%

## Security Considerations

### Threat Model

**Trusted**:
- Local system
- Serial port hardware
- Configuration files

**Untrusted**:
- TCP network (if bound to 0.0.0.0)
- TCP clients
- Data from serial port (RF can be spoofed)

### Security Features

**What rax25kb Does**:
- Binds only to specified addresses
- Validates configuration input
- Limits buffer sizes
- Handles malformed data gracefully

**What rax25kb Does NOT Do**:
- No authentication
- No encryption
- No access control
- No rate limiting
- No input sanitization for display

### Recommended Practices

1. **Network Binding**:
   - Use `127.0.0.1` for local-only access
   - Use firewall rules for remote access
   - Don't expose to untrusted networks

2. **File Permissions**:
   - Restrict config file to owner
   - Protect log files (may contain callsigns)
   - Protect PCAP files (contain packet data)

3. **Serial Port Access**:
   - Limit user access via groups (dialout)
   - Physical security of hardware

## Platform-Specific Implementation

### Cross-Platform Strategy

**Abstraction Layer**: `serialport` crate provides platform-independent API

**Platform Differences**:

| Feature | Linux | Windows | macOS |
|---------|-------|---------|-------|
| Serial devices | `/dev/ttyUSB*` | `COM*` | `/dev/cu.*` |
| Flow control | Full support | Full support | Full support |
| DTR/DSR | Native | Native | Limited |
| File paths | Unix style | Windows style | Unix style |
| Service mgmt | systemd | NSSM/Task Scheduler | launchd |

### Platform-Specific Code

**Flow Control**:
```rust
#[cfg(target_os = "windows")]
{ port_builder.flow_control(serialport::FlowControl::Hardware) }

#[cfg(not(target_os = "windows"))]
{ port_builder.flow_control(serialport::FlowControl::None) }
```

**Path Handling**:
- Rust's `std::path` handles platform differences
- Configuration uses native path separators
- No hardcoded path assumptions

## Extension Points

### Adding New Features

**Configuration Options**:
1. Add field to `Config` struct
2. Add parsing in `Config::from_file()`
3. Add CLI override in `apply_cli_overrides()`
4. Add default value
5. Document in help text

**KISS Commands**:
1. Add constant for command byte
2. Add parsing in `parse_kiss_frame()`
3. Add handling logic
4. Update documentation

**Protocol Support**:
1. Add new protocol parser
2. Add configuration flag
3. Integrate into frame processing
4. Add tests

**Output Formats**:
1. Add writer struct (like `PcapWriter`)
2. Integrate into frame processing
3. Add configuration option
4. Document usage

### Plugin Architecture (Future)

Currently not supported, but could be added:

**Potential Plugin Points**:
- Frame processors (modify frames in flight)
- Protocol parsers (new protocols beyond KISS/AX.25)
- Output formatters (new capture formats)
- Authentication handlers (validate clients)

**Implementation Approach**:
- Dynamic library loading
- Trait-based plugin interface
- Configuration-driven plugin loading
- Sandboxed plugin execution

## Design Patterns Used

### Patterns

1. **Builder Pattern**: Serial port configuration
2. **Strategy Pattern**: Flow control options (enum-based)
3. **Factory Pattern**: Logger creation
4. **Singleton**: PID file, PCAP file (per instance)
5. **Observer**: Logging system (fire-and-forget)
6. **Chain of Responsibility**: Frame processing pipeline

### Rust Idioms

1. **Ownership**: Clear ownership of resources
2. **Borrowing**: References where appropriate
3. **Arc/Mutex**: Shared mutable state
4. **Result/Option**: Error handling without exceptions
5. **Traits**: Polymorphic behavior (SerialPort)
6. **Enums**: Type-safe options

## Future Architecture Considerations

### Potential Improvements

1. **Async I/O**:
   - Use `tokio` for async operations
   - Better scalability for multiple connections
   - More complex implementation

2. **Multi-Client Support**:
   - Broadcast frames to multiple clients
   - Per-client filtering
   - Connection management

3. **Statistics**:
   - Packet counters
   - Error rates
   - Throughput metrics
   - Prometheus integration

4. **Authentication**:
   - API key support
   - TLS/SSL encryption
   - Client certificates

5. **Configuration Reload**:
   - SIGHUP handler
   - Dynamic reconfiguration
   - No restart required

6. **Protocol Multiplexing**:
   - Support multiple protocols simultaneously
   - Protocol auto-detection
   - Per-port configuration

### Architectural Constraints

**Must Maintain**:
- Cross-platform compatibility
- Low latency operation
- Simple deployment (single binary)
- Minimal runtime dependencies

**Can Change**:
- Internal threading model
- Buffer management strategy
- Output format details
- Processing pipeline

## Conclusion

The rax25kb architecture prioritizes:
- **Simplicity**: Easy to understand and maintain
- **Reliability**: Robust error handling
- **Performance**: Low overhead, high throughput
- **Portability**: Works across platforms

The design is intentionally conservative, using proven patterns and avoiding unnecessary complexity. This makes the codebase accessible to contributors while providing stable, predictable operation for users.

---

**Document Version**: 1.0  
**Last Updated**: December 20, 2025  
**Author**: rax25kb contributors