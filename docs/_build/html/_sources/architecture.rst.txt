================================================================================
Architecture Documentation
================================================================================

rax25kb - AX.25 KISS Bridge with Multi-Port Cross-Connect Support

:Version: 2.0.0
:Author: Kris Kirby, KE4AHR
:Date: December 2025

================================================================================
Overview
================================================================================

rax25kb is a multi-threaded KISS protocol bridge designed for amateur packet
radio systems. It provides flexible routing between serial TNCs (Terminal Node
Controllers) and TCP/IP networks with protocol translation, debugging, and
packet capture capabilities.

Core Capabilities
--------------------------------------------------------------------------------

* **Multi-Port Support**: Manage multiple serial TNCs simultaneously
* **Flexible Routing**: Create arbitrary cross-connects between endpoints
* **Protocol Translation**: Convert between Standard and Extended KISS
* **Hardware Workarounds**: Built-in fixes for buggy TNC firmware (PhilFlag)
* **Debugging Tools**: Frame parsing, hexdump, and PCAP capture
* **Raw Access**: Bypass KISS processing for TNC configuration

================================================================================
System Architecture
================================================================================

Component Hierarchy
--------------------------------------------------------------------------------

The system is organized into five major layers::

    ┌─────────────────────────────────────────────────────────────┐
    │                     Main Application                         │
    │  (Initialization, Configuration, Signal Handling)            │
    └─────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
    ┌─────────────────────────────────────────────────────────────┐
    │                 CrossConnectManager                          │
    │  (Connection Lifecycle, Thread Spawning)                     │
    └─────────────────────────────────────────────────────────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    ▼                           ▼
    ┌───────────────────────────┐   ┌───────────────────────────┐
    │   SerialPortManager       │   │   Connection Handlers     │
    │  (Serial Port Lifecycle)  │   │  (Data Flow Threads)      │
    └───────────────────────────┘   └───────────────────────────┘
                    │                           │
                    ▼                           ▼
    ┌───────────────────────────┐   ┌───────────────────────────┐
    │   Support Services        │   │   KISS Processing         │
    │  (Logger, PCAP, Config)   │   │  (Frame Buffer, Parser)   │
    └───────────────────────────┘   └───────────────────────────┘

Thread Model
--------------------------------------------------------------------------------

rax25kb uses a multi-threaded architecture for concurrent I/O operations:

**Main Thread**
  - Loads configuration
  - Initializes subsystems
  - Spawns connection threads
  - Sleeps in event loop

**Listener Threads** (one per TCP endpoint)
  - Accept incoming TCP connections
  - Spawn handler threads for each client
  - Never terminate (unless error)

**Handler Threads** (two per connection)
  - **Serial → TCP Thread**: Reads serial, filters/processes, writes TCP
  - **TCP → Serial Thread**: Reads TCP, modifies/processes, writes serial

**Bridge Threads** (two per serial-to-serial bridge)
  - **Port A → Port B Thread**: Translates KISS port numbers A→B
  - **Port B → Port A Thread**: Translates KISS port numbers B→A

Thread Communication
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Threads communicate through:

* **Arc<Mutex<T>>**: Shared serial port access
* **TcpStream.try_clone()**: Independent TCP read/write handles
* **No channels**: Direct I/O without message passing (simpler, lower latency)

Thread Safety
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* Serial ports wrapped in ``Arc<Mutex<>>`` for multi-thread access
* Logger uses ``Arc<Mutex<File>>`` for concurrent log writes
* PCAP writer uses ``Arc<Mutex<File>>`` for packet capture
* No shared mutable state between threads (except protected resources)

================================================================================
Data Flow Architecture
================================================================================

Serial-to-TCP Connection
--------------------------------------------------------------------------------

::

    Serial Port                                           TCP Client
    ┌─────────┐                                          ┌─────────┐
    │  TNC    │                                          │  App    │
    └────┬────┘                                          └────┬────┘
         │                                                    │
         │ Raw KISS frames                                   │
         ▼                                                    │
    ┌─────────────────────────────────────┐                 │
    │     Serial → TCP Thread              │                 │
    │  ┌──────────────────────────────┐   │                 │
    │  │ 1. Read bytes from serial    │   │                 │
    │  │ 2. KissFrameBuffer extracts  │   │                 │
    │  │    complete frames            │   │                 │
    │  │ 3. Filter by KISS port       │   │                 │
    │  │ 4. Apply PhilFlag (optional) │   │                 │
    │  │ 5. Parse/display (optional)  │   │                 │
    │  │ 6. Write to TCP              │───┼─────────────────┤
    │  └──────────────────────────────┘   │                 │
    └─────────────────────────────────────┘                 │
                                                             │
                                                             │
    ┌─────────────────────────────────────┐                 │
    │     TCP → Serial Thread              │                 │
    │  ┌──────────────────────────────┐   │                 │
    │  │ 1. Read bytes from TCP       │◄──┼─────────────────┤
    │  │ 2. KissFrameBuffer extracts  │   │                 │
    │  │    complete frames            │   │                 │
    │  │ 3. Modify KISS port number   │   │                 │
    │  │ 4. Apply PhilFlag (optional) │   │                 │
    │  │ 5. Parse/display (optional)  │   │                 │
    │  │ 6. Write to serial           │   │                 │
    │  └──────────────────────────────┘   │                 │
    └─────────────────────────────────────┘                 │
         │                                                    │
         ▼                                                    ▼
    ┌─────────┐                                          ┌─────────┐
    │  TNC    │                                          │  App    │
    └─────────┘                                          └─────────┘

Serial-to-Serial Bridge
--------------------------------------------------------------------------------

::

    Port A (TNC #1)                               Port B (TNC #2)
    ┌─────────┐                                   ┌─────────┐
    │ TNC A   │                                   │ TNC B   │
    └────┬────┘                                   └────┬────┘
         │                                             │
         │ KISS Port X                                 │ KISS Port Y
         ▼                                             │
    ┌──────────────────────────────┐                  │
    │   Port A → Port B Thread     │                  │
    │  ┌────────────────────────┐  │                  │
    │  │ 1. Read from Port A    │  │                  │
    │  │ 2. Filter Port X       │  │                  │
    │  │ 3. Translate X → Y     │  │                  │
    │  │ 4. Write to Port B     │──┼──────────────────┤
    │  └────────────────────────┘  │                  │
    └──────────────────────────────┘                  │
                                                       ▼
    ┌──────────────────────────────┐            ┌─────────┐
    │   Port B → Port A Thread     │            │ TNC B   │
    │  ┌────────────────────────┐  │            └─────────┘
    │  │ 1. Read from Port B    │◄─┼──────────────────┘
    │  │ 2. Filter Port Y       │  │
    │  │ 3. Translate Y → X     │  │
    │  │ 4. Write to Port A     │  │
    │  └────────────────────────┘  │
    └──────────────────────────────┘
         │
         ▼
    ┌─────────┐
    │ TNC A   │
    └─────────┘

================================================================================
Protocol Architecture
================================================================================

KISS Protocol Layer
--------------------------------------------------------------------------------

KISS (Keep It Simple Stupid) is a framing protocol for TNC communication.

Frame Structure
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

::

    ┌──────┬────────────┬──────────────────────────────────┬──────┐
    │ FEND │ Port + Cmd │          Data Payload            │ FEND │
    │ 0xC0 │   1 byte   │       Variable Length            │ 0xC0 │
    └──────┴────────────┴──────────────────────────────────┴──────┘
             │
             └─► ┌─────────────────────┐
                 │  7  6  5  4  3  2  1  0  │
                 ├─────────────┼───────────┤
                 │  Port (0-15) │ Cmd (0-15) │
                 └─────────────┴───────────┘

Byte Stuffing
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

KISS uses byte stuffing to escape special characters in the data payload:

* ``0xC0`` (FEND) in data → ``0xDB 0xDC`` (FESC TFEND)
* ``0xDB`` (FESC) in data → ``0xDB 0xDD`` (FESC TFESC)

KISS Commands
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

===============  =======  ================================================
Command          Value    Purpose
===============  =======  ================================================
Data Frame       0x00     Transmit/receive data (most common)
TXDelay          0x01     Set transmit delay
Persistence      0x02     Set persistence parameter (CSMA)
SlotTime         0x03     Set slot time
TXtail           0x04     Set transmit tail time
FullDuplex       0x05     Set full/half duplex mode
SetHardware      0x06     Set hardware-specific parameters
Return           0x0F     Exit KISS mode
===============  =======  ================================================

KISS Port Numbers
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* **Standard KISS**: Always uses port 0
* **Extended KISS**: Supports ports 0-15 for multi-TNC systems
* rax25kb translates between port numbers for routing

AX.25 Protocol Layer
--------------------------------------------------------------------------------

AX.25 is the amateur radio data link layer protocol (based on X.25).

Frame Structure
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

::

    ┌─────────────┬─────────────┬──────────────┬─────────┬─────┬──────┬─────┐
    │ Destination │   Source    │  Digipeaters │ Control │ PID │ Info │ FCS │
    │  (7 bytes)  │  (7 bytes)  │  (0-56 bytes)│(1 byte) │(opt)│ (var)│(2 B)│
    └─────────────┴─────────────┴──────────────┴─────────┴─────┴──────┴─────┘

Address Format
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Each address is 7 bytes::

    ┌─────────────────────────────────────────────────────────┐
    │  Byte 0-5: Callsign (ASCII << 1)                        │
    │  Byte 6:   SSID + Reserved + Extension Bit               │
    │            [x x x x S S S S R R C E]                     │
    │            S=SSID (0-15), R=Reserved, C=C-bit, E=Ext     │
    └─────────────────────────────────────────────────────────┘

Frame Types
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* **I-frames** (Information): Data transfer in connected mode
* **S-frames** (Supervisory): Flow control (RR, RNR, REJ)
* **U-frames** (Unnumbered): Connection control (SABM, DISC, UA, DM)
* **UI-frames** (Unnumbered Information): Connectionless data (beacons, APRS)

================================================================================
Configuration Architecture
================================================================================

Configuration File Format
--------------------------------------------------------------------------------

Simple key=value format with support for:

* Comments (lines starting with ``#``)
* Quoted values (``key="value with spaces"``)
* Hierarchical keys (``serial_port0000_baud=9600``)
* Boolean values (``true``, ``false``, ``yes``, ``no``, ``1``, ``0``)

Configuration Sections
--------------------------------------------------------------------------------

Global Settings
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

::

    log_level=5              # Syslog-style levels (0-9)
    logfile=/var/log/rax25kb.log
    pidfile=/var/run/rax25kb.pid
    log_to_console=true
    quiet_startup=false
    pcap_file=/tmp/capture.pcap

Serial Port Definitions
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

::

    serial_port0000=/dev/ttyUSB0
    serial_port0000_baud=9600
    serial_port0000_flow_control=none
    serial_port0000_stop_bits=1
    serial_port0000_parity=none
    serial_port0000_extended_kiss=false

Cross-Connect Definitions
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

::

    cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
    cross_connect0000_phil_flag=false
    cross_connect0000_dump=false
    cross_connect0000_parse_kiss=true
    cross_connect0000_dump_ax25=false
    cross_connect0000_raw_copy=false

Configuration Validation
--------------------------------------------------------------------------------

The configuration parser validates:

* Serial port IDs are 4 digits (0000-9999)
* Cross-connect IDs are 4 digits (0000-9999)
* KISS port numbers are 0-15
* Serial port IDs referenced in cross-connects exist
* TCP ports are valid (1-65535)
* At least one serial port is defined
* At least one cross-connect is defined (or auto-created)

================================================================================
Storage Architecture
================================================================================

Logging System
--------------------------------------------------------------------------------

**Log Levels** (syslog-style)::

    0 = EMERG    - System is unusable
    1 = ALERT    - Action must be taken immediately
    2 = CRIT     - Critical conditions
    3 = ERROR    - Error conditions
    4 = WARN     - Warning conditions
    5 = NOTICE   - Normal but significant (default)
    6 = INFO     - Informational
    7 = DEBUG    - Debug-level messages
    8 = TRACE    - Trace execution
    9 = VERBOSE  - Very detailed output

**Log Format**::

    [YYYY-MM-DD HH:MM:SS] [LEVEL] message

**Thread Safety**: All log writes protected by ``Arc<Mutex<File>>``

PCAP Capture System
--------------------------------------------------------------------------------

**File Format**: Standard PCAP (libpcap/tcpdump format)

**Link Type**: DLT_AX25_KISS (147)

**Packet Format**::

    Global Header (24 bytes, written once)
    ┌──────────────────────────────────────────┐
    │ Magic Number: 0xa1b2c3d4                 │
    │ Version: 2.4                              │
    │ Timezone: GMT                             │
    │ Timestamp Accuracy: 0                     │
    │ Snapshot Length: 65535                    │
    │ Link Type: 147 (AX25_KISS)               │
    └──────────────────────────────────────────┘

    Per-Packet Header (16 bytes per packet)
    ┌──────────────────────────────────────────┐
    │ Timestamp (seconds)                       │
    │ Timestamp (microseconds)                  │
    │ Captured Length                           │
    │ Original Length                           │
    └──────────────────────────────────────────┘
    Packet Data (variable length)

**Wireshark Compatibility**: Files can be opened directly in Wireshark for
protocol analysis and debugging.

================================================================================
Error Handling Architecture
================================================================================

Error Propagation
--------------------------------------------------------------------------------

* **Configuration errors**: Fatal, printed to stderr, exit(1)
* **Serial port open errors**: Fatal, logged and printed, exit(1)
* **TCP bind errors**: Fatal, logged and printed, exit(1)
* **Connection errors**: Non-fatal, logged, thread exits
* **I/O errors**: Non-fatal, logged, connection closes gracefully

Recovery Strategies
--------------------------------------------------------------------------------

**TCP Connection Failures**
  - Listener thread continues accepting new connections
  - Each connection is independent
  - Client disconnect does not affect other clients

**Serial Port Read Errors**
  - Log error and close connection
  - Serial port remains open for other connections
  - Timeout errors are handled with retry

**Serial Port Write Errors**
  - Log error and close connection
  - Other connections continue operating

Signal Handling
--------------------------------------------------------------------------------

**SIGINT (Ctrl+C)**
  - Graceful shutdown message
  - Immediate exit(0)
  - OS cleans up threads and file descriptors

**Future Enhancement**: Proper cleanup (close serial ports, flush logs)

================================================================================
Performance Considerations
================================================================================

Latency Optimization
--------------------------------------------------------------------------------

* **No buffering**: Data forwarded immediately upon receipt
* **Direct thread communication**: No message queues or channels
* **Minimal locking**: Locks held only during I/O operations
* **Zero-copy where possible**: Frame references instead of copies

Throughput Optimization
--------------------------------------------------------------------------------

* **1024-byte read buffers**: Balance between syscall overhead and memory
* **Frame batching**: Process all complete frames from a read
* **Parallel processing**: Multiple connections operate independently

Memory Management
--------------------------------------------------------------------------------

* **Small footprint**: ~1-2 MB resident memory per connection
* **No memory leaks**: Rust's ownership system prevents leaks
* **Bounded buffers**: Frame buffers limited to single frame size

Scalability Limits
--------------------------------------------------------------------------------

* **Serial ports**: Limited by OS (typically ~100-200 ports)
* **TCP connections**: Limited by file descriptors (default ~1024)
* **Threads**: 2 threads per connection (can support ~500 connections)
* **CPU**: Minimal CPU usage, mostly I/O bound

================================================================================
Security Considerations
================================================================================

Attack Surface
--------------------------------------------------------------------------------

**TCP Listeners**
  - Bind to specific interfaces to limit exposure
  - No authentication (intended for trusted networks)
  - No encryption (plain text protocol)

**Serial Ports**
  - Requires physical or USB access
  - No privilege separation

Recommendations
--------------------------------------------------------------------------------

* **Use firewall rules** to restrict TCP access to trusted networks
* **Run as dedicated user** with minimal privileges
* **Use VPN or SSH tunneling** for remote access
* **Monitor logs** for unusual activity
* **Validate PCAP files** before sharing (may contain sensitive traffic)

================================================================================
Future Architecture Considerations
================================================================================

Potential Enhancements
--------------------------------------------------------------------------------

* **Plugin system**: Loadable modules for custom processing
* **Web interface**: HTTP API for configuration and monitoring
* **Statistics**: Per-connection byte/frame counters
* **Reconnection**: Automatic TCP client reconnection
* **IPv6 support**: Full IPv6 addressing
* **TLS support**: Encrypted TCP connections
* **Multi-cast**: UDP multicast for monitoring
* **Hot reload**: Configuration changes without restart

Backward Compatibility
--------------------------------------------------------------------------------

* Configuration format is extensible
* New features use opt-in flags
* Default behavior remains unchanged
* Protocol compatibility maintained

================================================================================
References
================================================================================

* **AX.25 Specification**: ARRL AX.25 Link Access Protocol v2.2
* **KISS Protocol**: Mike Chepponis, K3MC and Phil Karn, KA9Q (1987)
* **Extended KISS**: Multi-port extension by various authors
* **PCAP Format**: libpcap/tcpdump documentation
* **Rust serialport**: https://crates.io/crates/serialport

================================================================================
Glossary
================================================================================

See ``glossary.rst`` for definitions of terms used in this document.

================================================================================
End of Architecture Documentation
================================================================================