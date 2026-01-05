# Changelog

All notable changes to rax25kb will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.7.3] - 2025-12-31

### Fixed
- **Compiler Warning** ✅
  - Added `#[allow(dead_code)]` attribute to `AgwClientInfo.connected_at` field
  - Field reserved for future use (connection tracking, statistics, session management)

### Changed
- src/main.rs: Line 763 - Added allow attribute with explanatory comment

### Technical Details
- Warning: Field `connected_at` is never read
- Field is initialized when client connects but not currently used
- Reserved for future features: connection duration tracking, idle timeout, statistics
- Suppressing warning is appropriate for forward-compatibility fields

## [1.7.2] - 2025-12-31

### Fixed
- **Compilation Errors** ✅
  - Fixed moved value error in `CrossConnectBridge::new()` by saving `agw_enable` before moving `config`
  - Fixed packed field unaligned reference in AGW frame logging by copying `header.port` to local variable
  - Fixed unused variable warning by renaming `agw_header` to `_agw_header` in `agw_to_kiss()`

### Technical Details
- **Error E0382:** `config` was moved before accessing `config.agw_enable`
  - Solution: Save `agw_enable` value before moving `config` struct
- **Error E0793:** Reference to packed field `header.port` creates unaligned reference
  - Solution: Copy field value to local variable before formatting
  - Packed structs are only aligned by one byte, causing undefined behavior
- **Warning:** Unused `agw_header` parameter in `agw_to_kiss()`
  - Solution: Prefix with underscore to indicate intentionally unused
  - Parameter kept for API consistency and future use

### Changed
- src/main.rs: Line 835 - Added `agw_enable` local variable
- src/main.rs: Line 1454 - Added `port` local variable for safe access
- src/main.rs: Line 1844 - Renamed parameter to `_agw_header`

## [1.7.1] - 2025-12-31

### Fixed
- **Compilation Errors** ✅
  - Removed duplicate `extract_callsign` function definition
  - Fixed `AgwClientInfo` vector initialization to not require `Clone` trait
  - Changed `vec![None; max_clients]` to `(0..max_clients).map(|_| None).collect()`
  - Resolved conflict with pre-existing `extract_callsign` function in codebase

### Changed
- **Source Code**
  - src/main.rs: 2,202 → 2,182 lines (removed duplicate function)
  - AGW client vector initialization now matches tcp_clients pattern for consistency

### Technical Details
- Issue: `TcpStream` doesn't implement `Clone`, preventing use of `vec![value; n]` syntax
- Solution: Use iterator pattern to create independent `None` values without cloning
- Maintains identical runtime behavior to original implementation
- All AGW v1.7.0 functionality preserved

## [1.7.0] - 2025-12-31

### Added - AGW (AGWPE) Protocol Support
- **AGW TCP Server** ✅
  - Full AGW Packet Engine protocol implementation
  - Default port 8000 (standard AGWPE port)
  - `agw_server_enable` global configuration (default: no)
  - `agw_server_address` configuration (default: "0.0.0.0")
  - `agw_server_port` configuration (default: 8000)
  - `agw_max_clients` configuration (default: 3)
  
- **Per-Bridge AGW Configuration** ✅
  - `agw_enable` per cross-connect (default: no)
  - `agw_port` parameter (0-255, default: 0)
  - Maps AGW port numbers to KISS ports
  
- **AGW Protocol Implementation** ✅
  - Port Information Request/Response ('G')
  - Port Capabilities Request/Response ('g')
  - Callsign Registration/Unregistration ('X'/'x')
  - Raw Data Transmission ('K')
  - Monitor Mode Enable/Disable ('M'/'m')
  - Unproto Information for monitors ('U')
  - 36-byte AGW header format
  - Variable-length data payloads
  
- **Protocol Translation** ✅
  - Bidirectional KISS ↔ AGW conversion
  - AX.25 address extraction (FROM/TO callsigns)
  - Automatic SSID handling
  - PhilFlag integration (AGW packets processed identically to KISS)
  
- **AGW Client Management** ✅
  - Multiple simultaneous AGW clients (up to agw_max_clients)
  - Per-client callsign registration
  - Per-client monitor mode state
  - Connection tracking with timestamps
  - Automatic cleanup on disconnect
  
- **Dual Protocol Operation** ✅
  - KISS and AGW can run simultaneously
  - Different ports (e.g., 8001 for KISS, 8000 for AGW)
  - Share same TNC hardware
  - Independent client management
  - Both protocols receive identical packets from TNC
  
- **Monitor Mode Support** ✅
  - AGW clients can enable monitor mode
  - Receive all packets, not just addressed to them
  - Useful for APRS monitoring and logging
  - Per-client toggle
  
### Changed
- **Data Structures**
  - Added `AgwHeader` struct (36-byte AGW frame header)
  - Added `AgwClientInfo` struct for client tracking
  - Added AGW fields to `Config` struct
  - Added AGW fields to `CrossConnect` struct
  - Added AGW fields to `CrossConnectBridge` struct
  
- **Source Code**
  - src/main.rs: 1,646 → 2,202 lines (+556 lines)
  - New AGW protocol functions (~225 lines)
  - New `start_agw_listener()` method (~210 lines)
  - Enhanced serial read threads with AGW broadcast
  
### Added - Helper Functions
- `extract_ax25_addresses()` - Parse AX.25 addresses
- `extract_callsign()` - Extract callsign with SSID
- `build_agw_frame()` - Build AGW protocol frame
- `kiss_to_agw()` - Convert KISS to AGW
- `agw_to_kiss()` - Convert AGW to KISS with PhilFlag
- `send_agw_frame()` - Send frame to client
- `handle_agw_port_info()` - Port info handler
- `handle_agw_capabilities()` - Capabilities handler
- `handle_agw_register()` - Registration handler
- `handle_agw_monitor_enable()` - Monitor enable handler
- `handle_agw_monitor_disable()` - Monitor disable handler
- `send_to_agw_clients()` - Broadcast to AGW clients

### PhilFlag Integration
- ✅ AGW packets processed through PhilFlag logic (TX and RX)
- ✅ Uses same `process_phil_flag_tcp_to_serial()` function as KISS
- ✅ Controlled by `phil_flag` configuration parameter
- ✅ Ensures consistent behavior across KISS and AGW protocols
- ✅ Both protocols receive corrected data from TNC

### Compatibility
- **Windows Applications:** YAAC, UI-View32, WinLink Express, AGWTracker, Outpost PM, BPQ32
- **Cross-Platform:** YAAC (Java-based)
- **Existing KISS Apps:** Unchanged, fully compatible
- ✅ Backward compatible with all v1.6.8 configurations
- ✅ AGW disabled by default (opt-in feature)
- ✅ All v1.6.8 features preserved

### Documentation
- New example: `doc/examples/agw-server.cfg`
- AGW implementation plan
- PhilFlag integration documentation

### Architecture
- Dual protocol support (KISS + AGW simultaneously)
- Per-bridge AGW enable/disable
- Global AGW server configuration
- Shared TNC access across protocols
- Independent client management per protocol

## [1.6.8] - 2025-12-30

### Added - Direwolf Alignment Features
- **Multiple TCP Clients Per Port** ✅
  - Support for multiple simultaneous TCP clients per cross-connect
  - `max_tcp_clients` global configuration parameter (default: 3)
  - Client slot management with automatic cleanup
  - Per-client connection tracking with timestamps
  - Connection rejection when maximum clients reached
  - Broadcast from serial port to all connected TCP clients
  
- **Channel Filtering** ✅
  - `kiss_chan` parameter for per-port channel filtering
  - Value: -1 (all channels, default) or 0-15 (specific channel)
  - Filters packets by KISS channel before sending to TCP clients
  - Improves efficiency for single-channel applications
  
- **Channel Remapping** ✅
  - Automatic channel translation for legacy applications
  - When `kiss_chan` is 0-15: maps internal channel to KISS channel 0 for applications
  - Outgoing: Internal channel N → KISS channel 0 (to application)
  - Incoming: KISS channel 0 → Internal channel N (to radio)
  - Enables legacy single-channel apps to work with multi-channel TNCs
  
- **KISSCOPY Feature** ✅
  - `kiss_copy` parameter (yes/no, default: no)
  - Broadcasts packets from one TCP client to all other connected clients
  - Useful for monitoring applications
  - Independent of serial port operation
  - Per-client source tracking to avoid loops

- **Serial Port Hierarchy** ✅
  - KISS port 0 designated as primary port for each serial device
  - Primary port controls all serial parameters (baud, flow control, etc.)
  - KISS ports 1-15 inherit parameters from port 0
  - Secondary ports share serial handle from primary (efficient R/W access)
  - Automatic primary port identification during configuration parsing
  - `is_primary_port` flag in CrossConnect structure
  
### Changed
- **Data Structures**
  - `tcp_client: Option<TcpStream>` → `tcp_clients: Vec<Option<TcpClientInfo>>`
  - Added `TcpClientInfo` struct with stream and connection timestamp
  - Added `kiss_chan: i32` to CrossConnect (-1 for all channels)
  - Added `kiss_copy: bool` to CrossConnect
  - Added `is_primary_port: bool` to CrossConnect
  - Added `max_tcp_clients: usize` to Config (global setting)
  
- **TCP Server Implementation**
  - Completely rewritten `start_tcp_listener()` for multi-client support
  - Accept loop now manages up to `max_tcp_clients` simultaneous connections
  - Serial read thread broadcasts to all connected TCP clients
  - Per-client read threads with independent buffers
  
- **TCP Client Mode**
  - Updated to use new `tcp_clients` vector structure
  - Uses index 0 of client vector for single connection
  - Maintains backward compatibility with reconnection logic
  
- **Bridge Creation**
  - Two-pass initialization: primary ports first, then secondary
  - Serial handle sharing between KISS ports on same device
  - `CrossConnectBridge::new()` accepts `shared_serial` parameter
  - Only primary ports open physical serial connections

### Added - Helper Functions
- `send_to_all_tcp_clients()` - Broadcasts data to all connected clients
- `broadcast_to_other_clients()` - KISSCOPY implementation
- `remap_kiss_channel_in()` - Channel remapping for incoming packets

### Configuration
- New global parameter: `max_tcp_clients` (default: 3)
- New per-cross-connect parameters:
  - `kiss_chan` (default: -1, all channels)
  - `kiss_copy` (default: no)
  
### Compatibility
- ✅ Backward compatible with v1.6.7 configurations
- ✅ Existing single-client configs work unchanged (defaults provide same behavior)
- ✅ All v1.6.7 features preserved (XKISS, PhilFlag, TCP client, etc.)
- ✅ New features are opt-in via configuration parameters

### Architecture
- Aligns KISS-over-TCP behavior with direwolf implementation
- Maintains rax25kb's unique features (XKISS, serial-to-serial, etc.)
- Enhanced scalability (3+ clients vs 1 in v1.6.7)
- Improved resource sharing through serial port hierarchy

### Documentation
- Implementation plan document created
- Status tracking document
- Updated version strings throughout codebase

## [1.6.7] - 2025-12-30T13:41+00:00

### Added
- **TCP Client Mode** ✅
  - `tcp_mode` parameter (server/client/none)
  - `tcp_server_address` parameter for client mode
  - `tcp_server_port` parameter for client mode
  - Automatic reconnection with exponential backoff
  - Connection state logging
  - `start_tcp_client()` method in CrossConnectBridge

- **XKISS RX Buffer** ✅
  - `XkissRxBuffer` struct with push/pop/flush operations
  - `xkiss_rx_buffer_size` configuration parameter
  - Configurable buffer size: 4KB minimum, 1MB maximum, 16KB default
  - Integrated with XKISS polling mode
  - Automatic buffer flush on poll cycle
  - Buffer overflow detection and logging
  - Per-packet buffering for polling mode

- **TCP-to-TCP Support** (with safety flags) ⚠️
  - `tcp_to_tcp_dangerous` flag (required for TCP-to-TCP)
  - `tcp_to_tcp_also_dangerous` flag (disables KISS validation)
  - KISS packet validation for TCP-to-TCP connections
  - Warning logging for dangerous mode activation
  - `is_kiss_packet()` validation function

- **Packet Reframing** (experimental)
  - `reframe_large_packets` configuration flag
  - `reframe_large_packet()` function for AX.25 packet splitting
  - `estimate_philflag_size()` for size prediction
  - Automatic splitting of packets exceeding 255 bytes after PhilFlag
  - Conservative chunk size handling (220 bytes)

- **Comprehensive Documentation**
  - 40+ page Cross-Connect Guide (`doc/CROSS-CONNECT-GUIDE.md`)
  - Detailed security warnings for TCP-to-TCP
  - Configuration examples for all modes
  - Troubleshooting section
  - Performance considerations
  - Best practices guide

### Changed
- **Version**: 1.6.6 → 1.6.7 throughout all files
- **Bridge Starting Logic**: Now handles Server/Client/None TCP modes
- **Serial Read Thread**: Integrated with XKISS RX buffer
- **Packet Output**: Conditional buffering vs. immediate send based on polling mode
- **CrossConnectBridge**: Added `xkiss_rx_buffer` field
- **Configuration Parsing**: All new parameters validated and parsed

### Implementation Details
- TCP client includes automatic reconnection with exponential backoff (1-60 seconds)
- XKISS buffer stores packets until poll timer fires
- Polling thread flushes buffer every `xkiss_poll_timer_ms` milliseconds
- TCP-to-TCP enforces KISS packet validation unless `also_dangerous` flag set
- Packet reframing parses AX.25 headers and splits info field

### Security
- **WARNING**: TCP-to-TCP mode bypasses hardware serial isolation
- TCP-to-TCP requires explicit `tcp_to_tcp_dangerous=true` flag
- KISS validation enforced by default in TCP-to-TCP mode
- Comprehensive security documentation in Cross-Connect Guide
- Logging of all dangerous mode activations

### Testing Status
- ✅ Configuration parsing: Complete
- ✅ XKISS buffer operations: Implemented
- ✅ TCP client connection: Implemented
- ⏳ TCP client reconnection: Implemented, needs real-world testing
- ⏳ TCP-to-TCP routing: Partial, needs integration testing
- ⏳ Packet reframing: Implemented, needs hardware testing
- ⏳ Buffer overflow handling: Implemented, needs stress testing

### Known Limitations
- TCP-to-TCP requires careful configuration (see documentation)
- Packet reframing may confuse some applications
- Buffer overflow drops packets (logged but not queued)
- No fragment reassembly for reframed packets

## [1.6.6] - 2025-12-30T07:42+00:00

### Added
- 7-bit serial mode support (DataBits enum: Seven, Eight)
- Explicit 7N1 serial mode (7 data bits, no parity, 1 stop bit)
- 7E1/7O1 modes for 7-bit serial with parity
- XKISS checksum support (optional, default: OFF)
  - `xkiss_checksum` configuration parameter
  - `calculate_xkiss_checksum()` function
  - `verify_xkiss_checksum()` function
- XKISS polling mode support (renamed from "polled mode")
  - `xkiss_polling` configuration parameter (default: OFF)
  - `xkiss_poll_timer_ms` configuration parameter (default: 100ms)
  - Poll response handling per XKISS specification
- TNC emulation capability documentation
- Configuration file notes:
  - MFJ-1278 TNCs default to 7E1 at 2400 bps
  - Polling mode terminology clarification
  - TNC emulation capability note
- GPLv3 LICENSE file in project root
- Comprehensive cliff notes in env/cliffnotes.md for AI continuation

### Changed
- **Author**: Updated to "Kris Kirby, KE4AHR" in all files
- **Copyright**: Updated to "Copyright (C)2025-2026 Kris Kirby, KE4AHR" in all files
- **SPDX**: Added "SPDX-License-Identifier: GPL-3.0-or-later" to all source files
- **Version**: 1.6.5 → 1.6.6 throughout all files
- **Terminology**: "polled mode" → "polling mode" for XKISS (aligns with spec)
- Serial port configuration now enforces 8N1 when KISS/XKISS is active
  - KISS specification requires 8 data bits, no parity, 1 stop bit
  - Parity and data_bits config ignored in KISS/XKISS mode
  - Raw copy mode respects all serial port configuration
- CrossConnect struct expanded with new fields:
  - `data_bits: DataBits`
  - `xkiss_checksum: bool`
  - `xkiss_polling: bool`
  - `xkiss_poll_timer_ms: u64`

### Fixed
- Serial port modes now properly support 7-bit configurations
- KISS/XKISS mode enforcement prevents invalid serial configurations

### Documentation
- Added comprehensive cliff notes for project continuation
- Updated all version references
- Added MFJ-1278 compatibility notes
- Clarified XKISS polling terminology
- Added TNC emulation mode documentation

## [1.6.5] - 2025-12-30T04:09+00:00

### Added
- Multiple serial port support via cross-connect configuration
- Cross-connect functionality (cross_connect0000 to cross_connect9999)
- Serial-to-serial bridging capability
- Serial-to-TCP bridging with single connection per port
- KISS to Extended KISS (XKISS) translation support
- Bidirectional KISS/XKISS port number translation
- Configuration items for cross_connectXXXX (0000-9999 range)
- Support for connecting standard KISS TNC Port 0 to XKISS ports
- `#[allow(dead_code)]` attribute to suppress KISS_TFESC warning
- Comprehensive Sphinx documentation system
- Windows-specific usage examples in documentation
- Man page for rax25kb command (section 1)
- Man page for rax25kb.cfg configuration file (section 5)
- ARCHITECTURE.md document describing system design
- INSTALL.md with installation instructions
- TREE.md file tree documentation
- Enhanced configuration file parsing for cross-connect objects
- Per-cross-connect configuration options

### Changed
- Updated support URL from outpostpm.org to https://github.com/ke4ahr/rax25kb/
- Refactored KissBridge to CrossConnectBridge for multi-port support
- Modified configuration structure to support multiple cross-connects
- Enhanced TCP listener to support single connection per cross-connect
- Improved logging with bridge ID prefixes
- Updated help text to reflect new cross-connect capabilities
- CLI overrides now apply to first cross-connect (cross_connect0000)

### Fixed
- Dead code warning for KISS_TFESC constant

### Removed
- Legacy single-port limitation

## [1.6.4] - Previous Release
### Note
- Prior changelog entries not available in this version

