================================================================================
Changelog
================================================================================

rax25kb - AX.25 KISS Bridge with Multi-Port Cross-Connect Support

All notable changes to this project will be documented in this file.

The format is based on `Keep a Changelog <https://keepachangelog.com/>`_,
and this project adheres to `Semantic Versioning <https://semver.org/>`_.

================================================================================
[Unreleased]
================================================================================

Changes that have been committed but not yet released.

Added
--------------------------------------------------------------------------------

* None yet

Changed
--------------------------------------------------------------------------------

* None yet

Fixed
--------------------------------------------------------------------------------

* Properly utilize ``dump_ax25`` configuration flag in frame display
* Function ``parse_kiss_frame_static()`` now accepts ``dump_ax25`` parameter

Deprecated
--------------------------------------------------------------------------------

* None yet

Removed
--------------------------------------------------------------------------------

* None yet

Security
--------------------------------------------------------------------------------

* None yet

================================================================================
[2.0.0] - 2025-12-26
================================================================================

Major rewrite with multi-port support and improved architecture.

Added
--------------------------------------------------------------------------------

* **Multi-port serial support**: Manage multiple TNCs simultaneously
* **Cross-connect system**: Flexible routing between any endpoints
* **Serial-to-serial bridging**: Connect TNCs together with port translation
* **Raw copy mode**: Direct byte passthrough for TNC configuration
* **PCAP capture**: Wireshark-compatible packet capture (DLT_AX25_KISS)
* **Frame parsing**: Display KISS and AX.25 frame details
* **Hexdump mode**: Display frames in hexadecimal format
* **PhilFlag support**: Hardware workarounds for buggy TNCs
* **Extended KISS translation**: Convert between Standard and Extended KISS
* **Per-connection features**: Enable/disable features per cross-connect
* **Configuration validation**: Comprehensive error checking and helpful messages
* **Structured logging**: Syslog-style log levels (0-9)
* **PID file support**: For daemon management
* **Quiet mode**: Suppress startup banner
* **Thread-safe architecture**: Proper concurrent serial port access
* **Signal handling**: Graceful shutdown on SIGINT/SIGTERM
* **Comprehensive documentation**: Architecture, building, contributing guides

Changed
--------------------------------------------------------------------------------

* **Configuration format**: Now uses hierarchical key=value format
* **Architecture**: Complete rewrite with modular design
* **Threading model**: One thread per connection direction
* **Error handling**: Better error messages and recovery
* **Code organization**: Split into 5 logical parts for maintainability
* **Performance**: Reduced latency with direct thread communication

Fixed
--------------------------------------------------------------------------------

* **Frame boundary detection**: Proper KISS frame extraction
* **Byte stuffing**: Correct handling of FESC sequences
* **Port number translation**: Reliable KISS port modification
* **Serial timeout handling**: Proper recovery from timeout errors
* **TCP connection cleanup**: Graceful client disconnect handling
* **Memory leaks**: Eliminated through Rust's ownership system

Security
--------------------------------------------------------------------------------

* **Privilege separation**: Can run as non-root user (with proper permissions)
* **No authentication**: Intended for trusted networks only (document clearly)

================================================================================
[1.5.1] - 2024-11-15
================================================================================

Maintenance release with bug fixes.

Fixed
--------------------------------------------------------------------------------

* Serial port enumeration on Windows
* Configuration file parsing edge cases
* Memory usage optimization

================================================================================
[1.5.0] - 2024-09-20
================================================================================

Feature release with Extended KISS support.

Added
--------------------------------------------------------------------------------

* Extended KISS protocol support
* Configurable KISS port numbers
* Basic logging to file

Changed
--------------------------------------------------------------------------------

* Improved serial port error handling
* Updated dependencies to latest versions

Fixed
--------------------------------------------------------------------------------

* Race condition in TCP accept loop
* Buffer overflow in frame processing

================================================================================
[1.4.2] - 2024-06-10
================================================================================

Bug fix release.

Fixed
--------------------------------------------------------------------------------

* Crash when TNC sends malformed frames
* Incorrect baud rate calculation for some speeds
* Configuration file parsing with quoted values

================================================================================
[1.4.1] - 2024-04-05
================================================================================

Minor bug fix release.

Fixed
--------------------------------------------------------------------------------

* TCP connection not closing properly on client disconnect
* Serial port not releasing on program exit

Changed
--------------------------------------------------------------------------------

* Default log level changed to INFO

================================================================================
[1.4.0] - 2024-02-14
================================================================================

Feature release with improved protocol handling.

Added
--------------------------------------------------------------------------------

* KISS byte stuffing (FESC/TFEND/TFESC)
* Frame validation before forwarding
* Statistics logging (frame counts, errors)

Changed
--------------------------------------------------------------------------------

* Refactored main loop for better readability
* Improved error messages

Fixed
--------------------------------------------------------------------------------

* KISS frame detection with multiple FENDs
* Buffer not being cleared between frames

================================================================================
[1.3.0] - 2023-12-01
================================================================================

Added
--------------------------------------------------------------------------------

* Support for hardware flow control (RTS/CTS)
* Configurable stop bits and parity
* Command-line configuration file option (-c)

Changed
--------------------------------------------------------------------------------

* Configuration file format (backward compatible)
* Serial timeout handling

Fixed
--------------------------------------------------------------------------------

* High CPU usage on idle connections
* Serial port locking issues on Linux

================================================================================
[1.2.0] - 2023-09-18
================================================================================

Added
--------------------------------------------------------------------------------

* PhilFlag support for TASCO modems
* Kenwood TS-2000 compatibility fixes
* Verbose logging mode

Fixed
--------------------------------------------------------------------------------

* Frame corruption with certain TNCs
* TCP buffer handling

================================================================================
[1.1.0] - 2023-07-05
================================================================================

Added
--------------------------------------------------------------------------------

* Windows support (tested on Windows 10/11)
* macOS support (tested on macOS 12+)
* Cross-platform serial port handling

Changed
--------------------------------------------------------------------------------

* Serial port library to serialport-rs
* Improved error handling

Fixed
--------------------------------------------------------------------------------

* Serial port detection on macOS
* Configuration file path handling on Windows

================================================================================
[1.0.0] - 2023-05-01
================================================================================

Initial release.

Added
--------------------------------------------------------------------------------

* Basic KISS bridge functionality
* Single serial port support
* Single TCP listener
* Linux support
* Simple configuration file format
* Basic logging to stdout

Features
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* Serial port to TCP KISS bridge
* Configurable baud rate
* Basic frame forwarding
* KISS FEND frame detection

Limitations
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* No frame validation
* No byte stuffing support
* Single connection only
* Linux only
* No logging to file
* No daemon mode

================================================================================
Version History Summary
================================================================================

* **2.0.0** - Complete rewrite, multi-port support, major feature additions
* **1.5.x** - Extended KISS support, logging improvements
* **1.4.x** - Protocol fixes, better error handling
* **1.3.x** - Hardware flow control, configuration improvements
* **1.2.x** - PhilFlag support, TNC compatibility fixes
* **1.1.x** - Cross-platform support (Windows, macOS)
* **1.0.0** - Initial release (Linux only, basic functionality)

================================================================================
Upgrade Notes
================================================================================

Upgrading from 1.x to 2.0
--------------------------------------------------------------------------------

**Breaking Changes**

* Configuration file format has changed
* Command-line arguments have changed (removed some options)
* Old configuration files will not work without modification

**Migration Steps**

1. **Backup your old configuration**::

    cp rax25kb.conf rax25kb.conf.old

2. **Create new configuration file**

   Use the new format (see examples/rax25kb.cfg)::

    # Old format (1.x)
    serial_port=/dev/ttyUSB0
    baud_rate=9600
    tcp_port=8001

    # New format (2.0)
    serial_port0000=/dev/ttyUSB0
    serial_port0000_baud=9600
    cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001

3. **Test new configuration**::

    rax25kb -c rax25kb.cfg

4. **Update any scripts or systemd units** that reference old options

**New Capabilities**

* Multiple serial ports now supported
* Serial-to-serial bridging possible
* Per-connection feature flags
* Much better logging and debugging

**Removed Features**

* None - all 1.x features are still available in 2.0

================================================================================
Deprecation Policy
================================================================================

Features marked as deprecated will:

* Continue to work for at least 2 minor versions
* Display deprecation warnings when used
* Be documented in changelog with migration path
* Be removed in next major version

Example: Feature deprecated in 2.1.0 will be removed in 3.0.0

================================================================================
Contributing
================================================================================

When adding changelog entries:

1. Add to **[Unreleased]** section
2. Use past tense ("Added feature" not "Add feature")  
3. Be specific and concise
4. Reference issue/PR numbers when applicable
5. Group related changes together

See ``contributing.rst`` for detailed guidelines.

================================================================================
Links
================================================================================

* **Repository**: https://github.com/ke4ahr/rax25kb
* **Issue Tracker**: https://github.com/ke4ahr/rax25kb/issues
* **Releases**: https://github.com/ke4ahr/rax25kb/releases
* **Documentation**: https://github.com/ke4ahr/rax25kb/tree/main/docs

================================================================================
End of Changelog
================================================================================