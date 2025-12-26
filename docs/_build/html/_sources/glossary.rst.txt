================================================================================
Glossary
================================================================================

rax25kb - AX.25 KISS Bridge with Multi-Port Cross-Connect Support

:Version: 2.0.0
:Author: Kris Kirby, KE4AHR
:Date: December 2025

This glossary defines terms used throughout the rax25kb documentation and
source code.

================================================================================
A
================================================================================

**AFSK** (Audio Frequency-Shift Keying)
    Modulation method used in amateur packet radio. Data is encoded as audio
    tones (typically 1200 baud Bell 202 or 9600 baud G3RUH).

**APRS** (Automatic Packet Reporting System)
    Amateur radio-based system for real-time tactical digital communications,
    typically using UI frames on 144.390 MHz in North America.

**AX.25**
    Amateur radio data link layer protocol based on X.25. Defines frame
    structure, addressing, and connection management for packet radio.
    See: ``AX.25 Link Access Protocol v2.2``

**Arc** (Atomic Reference Counted)
    Rust type for thread-safe reference counting. Allows multiple owners of
    the same data with automatic cleanup.

================================================================================
B
================================================================================

**Baud Rate**
    Symbol rate of serial communication in symbols per second. Common rates:
    1200, 9600, 19200, 38400, 57600, 115200. Not the same as bits per second
    in all cases.

**Bridge**
    Software component that forwards data between different interfaces or
    networks without modifying the content.

**Byte Stuffing**
    Technique used in KISS protocol to escape special characters in data
    payload. FEND (0xC0) becomes FESC TFEND (0xDB 0xDC).

================================================================================
C
================================================================================

**Callsign**
    Unique identifier assigned to amateur radio operators (e.g., KE4AHR,
    N0CALL). Used as addresses in AX.25 frames.

**Command Byte**
    Second byte in KISS frame containing port number (upper 4 bits) and
    command (lower 4 bits).

**Connected Mode**
    AX.25 operation mode where a connection is established before data
    transfer (like TCP). Uses I-frames, S-frames, and U-frames.

**Control Byte**
    Byte in AX.25 frame that indicates frame type and contains sequence
    numbers for flow control.

**Cross-Connect**
    Configuration that routes data between two endpoints (e.g., serial port
    to TCP socket, or serial port to serial port).

**CSMA** (Carrier Sense Multiple Access)
    Medium access control method. Stations listen before transmitting to
    avoid collisions.

================================================================================
D
================================================================================

**Digipeater** (Digital Repeater)
    Station that retransmits packets to extend range. Can handle up to 8
    digipeater addresses in AX.25 frame.

**DLT** (Data Link Type)
    PCAP file format identifier for link layer protocol. rax25kb uses
    DLT_AX25_KISS (147).

**DTR/DSR** (Data Terminal Ready / Data Set Ready)
    Serial port hardware flow control signals. Less common than RTS/CTS.

================================================================================
E
================================================================================

**Endpoint**
    Source or destination in a cross-connect. Can be a serial port with KISS
    port number or a TCP socket address.

**Extended KISS**
    Enhancement to standard KISS that supports port numbers 0-15, allowing
    a single serial connection to address multiple virtual TNCs.

================================================================================
F
================================================================================

**FCS** (Frame Check Sequence)
    Error detection code at end of AX.25 frame. 16-bit CRC usually calculated
    by TNC hardware.

**FEND** (Frame End)
    KISS protocol special byte (0xC0) that marks start and end of frame.

**FESC** (Frame Escape)
    KISS protocol special byte (0xDB) that begins an escape sequence.

**Flow Control**
    Method to prevent buffer overflow in serial communication:
    
    * None - No flow control
    * Software (XON/XOFF) - Uses special characters
    * Hardware (RTS/CTS) - Uses dedicated wires
    * DTR/DSR - Alternative hardware method

================================================================================
G
================================================================================

**G3RUH**
    9600 baud modulation scheme designed by James Miller G3RUH. Used for
    higher-speed packet radio.

================================================================================
I
================================================================================

**I-frame** (Information Frame)
    AX.25 frame type that carries user data in connected mode. Includes
    sequence numbers for reliable delivery.

================================================================================
K
================================================================================

**KISS** (Keep It Simple Stupid)
    Protocol for communicating with TNCs over serial connection. Designed by
    Mike Chepponis (K3MC) and Phil Karn (KA9Q) in 1987.

**KISS Mode**
    TNC operating mode where the TNC acts as a simple data pump, passing
    frames transparently without interpretation.

**KISS Port**
    Number (0-15) in Extended KISS that identifies which virtual TNC or
    radio port a frame is for.

================================================================================
L
================================================================================

**Link Layer**
    OSI layer 2 - responsible for reliable data transfer between adjacent
    nodes. AX.25 operates at this layer.

**LTO** (Link-Time Optimization)
    Compiler optimization that operates across entire program. Produces
    smaller, faster binaries at cost of longer compile time.

================================================================================
M
================================================================================

**Modulation**
    Process of encoding digital data onto analog carrier. Common types in
    packet radio: AFSK (1200 baud), G3RUH (9600 baud).

**Mutex** (Mutual Exclusion)
    Synchronization primitive that ensures only one thread accesses a
    resource at a time.

================================================================================
N
================================================================================

**Node**
    Station in packet radio network. Can be end station or digipeater.

================================================================================
P
================================================================================

**Packet Radio**
    Method of transmitting data over amateur radio using digital protocols
    (usually AX.25).

**Parity**
    Error detection method for serial communication:
    
    * None - No parity bit (most common: 8N1)
    * Even - Parity bit makes bit count even
    * Odd - Parity bit makes bit count odd

**PCAP** (Packet Capture)
    File format for capturing network packets. Created by libpcap/tcpdump,
    used by Wireshark.

**PhilFlag**
    Workaround for KISS protocol bugs in TASCO modem chipsets and Kenwood
    TS-2000 TNCs. Escapes improperly handled FEND bytes.

**PID** (Protocol Identifier)
    Byte in AX.25 frame indicating upper layer protocol. Common values:
    0xF0 (no layer 3), 0x01 (ISO 8208/X.25), 0xCC (IP).

================================================================================
R
================================================================================

**Raw Copy Mode**
    Operating mode that bypasses KISS processing and copies bytes directly
    between endpoints. Useful for TNC configuration.

**RTS/CTS** (Request To Send / Clear To Send)
    Serial port hardware flow control signals. Most common hardware flow
    control method.

================================================================================
S
================================================================================

**S-frame** (Supervisory Frame)
    AX.25 frame type for flow control and acknowledgment. Types: RR (Receive
    Ready), RNR (Receive Not Ready), REJ (Reject).

**SABM** (Set Asynchronous Balanced Mode)
    AX.25 command to establish connected-mode connection.

**Serial Port**
    Physical interface for serial communication. Can be native RS-232, USB-
    to-serial adapter, or direct TNC USB connection.

**SSID** (Secondary Station Identifier)
    Number (0-15) appended to callsign to identify multiple stations using
    same callsign. Example: N0CALL-5.

**Standard KISS**
    Original KISS protocol supporting only port 0. Most TNCs use this.

**Stop Bits**
    Signaling bits marking end of byte in serial communication:
    
    * 1 stop bit - Most common
    * 2 stop bits - Required by some equipment (e.g., Kenwood TS-2000)

================================================================================
T
================================================================================

**TASCO**
    Modem chipset manufacturer whose KISS implementation has known bugs.
    PhilFlag workaround addresses these issues.

**TFEND** (Transposed Frame End)
    KISS escape sequence byte (0xDC). The sequence FESC TFEND represents a
    FEND in the data payload.

**TFESC** (Transposed Frame Escape)
    KISS escape sequence byte (0xDD). The sequence FESC TFESC represents a
    FESC in the data payload.

**TNC** (Terminal Node Controller)
    Device that interfaces between computer and radio for packet
    communication. Handles modulation/demodulation and AX.25 protocol.

**TCP** (Transmission Control Protocol)
    Internet protocol providing reliable, ordered byte stream. Used by
    rax25kb for network connections.

================================================================================
U
================================================================================

**U-frame** (Unnumbered Frame)
    AX.25 frame type for connection control. Examples: SABM (connect),
    DISC (disconnect), UA (acknowledge), DM (disconnected mode).

**UI Frame** (Unnumbered Information)
    Special AX.25 frame type for connectionless data transfer. Used for
    beacons, APRS, and broadcast messages.

**Unconnected Mode**
    AX.25 operation mode where data is sent without establishing connection
    (like UDP). Uses UI frames only.

================================================================================
V
================================================================================

**VHF** (Very High Frequency)
    Radio frequency range 30-300 MHz. Most packet radio operates on VHF
    (144-148 MHz amateur band).

================================================================================
W
================================================================================

**Wireshark**
    Network protocol analyzer. Can open PCAP files generated by rax25kb to
    inspect AX.25 frames.

================================================================================
X
================================================================================

**X.25**
    ITU-T standard network protocol. AX.25 is derived from X.25 layer 2,
    adapted for amateur radio use.

**XON/XOFF**
    Software flow control method using special characters: XON (0x11)
    resumes transmission, XOFF (0x13) pauses transmission.

================================================================================
Protocol Values
================================================================================

KISS Special Bytes
--------------------------------------------------------------------------------

========  ======  ==================================================
Name      Value   Description
========  ======  ==================================================
FEND      0xC0    Frame End delimiter
FESC      0xDB    Frame Escape (starts escape sequence)
TFEND     0xDC    Transposed FEND (after FESC)
TFESC     0xDD    Transposed FESC (after FESC)
========  ======  ==================================================

KISS Commands
--------------------------------------------------------------------------------

========  ======  ==================================================
Command   Value   Description
========  ======  ==================================================
Data      0x00    Data frame (transmit/receive)
TXDelay   0x01    Set transmit delay
P         0x02    Set persistence parameter
SlotTime  0x03    Set slot time
TXtail    0x04    Set transmit tail
FullDup   0x05    Set full/half duplex
SetHW     0x06    Set hardware parameters
Return    0x0F    Exit KISS mode
========  ======  ==================================================

AX.25 Frame Types
--------------------------------------------------------------------------------

=========  ==============  ===========================================
Type       Control Bits    Description
=========  ==============  ===========================================
I-frame    xxxxxxx0        Information (data transfer)
S-frame    xxxxxx01        Supervisory (flow control)
U-frame    xxxxxx11        Unnumbered (connection control)
UI-frame   00000011        Unnumbered Information (connectionless)
=========  ==============  ===========================================

Common AX.25 PID Values
--------------------------------------------------------------------------------

======  ================================================================
Value   Protocol
======  ================================================================
0x01    ISO 8208/CCITT X.25 PLP
0x06    Compressed TCP/IP packet (Van Jacobson)
0x07    Uncompressed TCP/IP packet (Van Jacobson)
0x08    Segmentation fragment
0xC3    TEXNET datagram protocol
0xC4    Link Quality Protocol
0xCA    Appletalk
0xCB    Appletalk ARP
0xCC    ARPA Internet Protocol (IP)
0xCD    ARPA Address Resolution Protocol (ARP)
0xCE    FlexNet
0xCF    NET/ROM
0xF0    No layer 3 protocol (most common)
0xFF    Escape character (next octet is PID)
======  ================================================================

Serial Port Settings
--------------------------------------------------------------------------------

Common Baud Rates
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* 1200 - Bell 202 AFSK (most common for VHF)
* 9600 - G3RUH (high-speed VHF)
* 19200 - Direct TNC connection
* 38400 - Fast direct connection
* 57600 - USB TNC connection
* 115200 - High-speed USB TNC

Common Settings
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* 8N1 - 8 data bits, No parity, 1 stop bit (most common)
* 8N2 - 8 data bits, No parity, 2 stop bits (Kenwood TS-2000)

TCP Port Numbers
--------------------------------------------------------------------------------

Common Ports for Packet Radio
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* 8001 - KISS protocol
* 6300-6310 - AGW Packet Engine ports
* 8100-8199 - Custom application ports

================================================================================
Abbreviations
================================================================================

Common Ham Radio and Networking Terms
--------------------------------------------------------------------------------

=======  ========================================================
Abbrev   Full Term
=======  ========================================================
73       Best regards (ham radio sign-off)
88       Love and kisses (ham radio sign-off)
ACK      Acknowledgment
ARP      Address Resolution Protocol
BBS      Bulletin Board System
CRC      Cyclic Redundancy Check
CQ       General call to all stations
DTE      Data Terminal Equipment
DCE      Data Communications Equipment
DE       This is (from) - used before callsign
IP       Internet Protocol
ISS      International Space Station
MHz      Megahertz
MTU      Maximum Transmission Unit
NIC      Network Interface Card
QSL      Acknowledgment of receipt
QSO      Contact/conversation between stations
QTH      Location
RF       Radio Frequency
RS232    Serial communication standard
RX       Receive
SSID     Secondary Station Identifier (or Service Set ID in WiFi)
TCP      Transmission Control Protocol
TNC      Terminal Node Controller
TX       Transmit
UDP      User Datagram Protocol
USB      Universal Serial Bus (or Upper Side Band in RF)
VHF      Very High Frequency
73       Best regards
=======  ========================================================

================================================================================
File Extensions
================================================================================

=======  ========================================================
Ext      Description
=======  ========================================================
.cfg     Configuration file
.log     Log file
.pcap    Packet capture file
.rs      Rust source file
.toml    Tom's Obvious Minimal Language (Cargo config)
.rst     reStructuredText documentation
.md      Markdown documentation
=======  ========================================================

================================================================================
Environment Variables
================================================================================

Rust and Cargo
--------------------------------------------------------------------------------

================  ====================================================
Variable          Purpose
================  ====================================================
CARGO_HOME        Cargo installation directory
RUSTUP_HOME       Rustup installation directory
RUST_BACKTRACE    Enable Rust panic backtraces (0, 1, full)
RUST_LOG          Set logging level for Rust programs
PATH              Must include ~/.cargo/bin for cargo commands
================  ====================================================

rax25kb Specific
--------------------------------------------------------------------------------

Currently rax25kb does not use environment variables. All configuration is
via configuration file.

================================================================================
References
================================================================================

Standards and Specifications
--------------------------------------------------------------------------------

* **AX.25 v2.2**: ARRL AX.25 Link Access Protocol v2.2 Specification
* **KISS**: Original spec by K3MC and KA9Q (1987)
* **X.25**: ITU-T Recommendation X.25
* **RS-232**: TIA/EIA-232 serial communication standard
* **PCAP**: libpcap file format documentation

Related Software
--------------------------------------------------------------------------------

* **direwolf**: Software TNC for Linux/Windows/macOS
* **soundmodem**: Linux soundcard modem
* **AGW Packet Engine**: Windows packet radio software
* **Xastir**: APRS client for Linux/Unix
* **YAAC**: Yet Another APRS Client (Java)

Organizations
--------------------------------------------------------------------------------

* **ARRL**: American Radio Relay League
* **TAPR**: Tucson Amateur Packet Radio
* **ITU**: International Telecommunication Union

================================================================================
Further Reading
================================================================================

Books
--------------------------------------------------------------------------------

* "Amateur Radio Techniques" by Pat Hawker, G3VA
* "The ARRL Handbook for Radio Communications"
* "Practical Packet Radio" by Stan Horzepa, WA1LOU

Online Resources
--------------------------------------------------------------------------------

* **TAPR**: https://www.tapr.org/
* **ARRL Digital Communications**: https://www.arrl.org/digital
* **Packet Radio History**: https://en.wikipedia.org/wiki/Packet_radio

================================================================================
Contributing to This Glossary
================================================================================

To add or modify terms:

1. Keep definitions concise (1-3 sentences)
2. Include relevant cross-references
3. Use consistent formatting
4. Add acronyms to abbreviations section
5. Sort terms alphabetically within sections

See ``contributing.rst`` for general contribution guidelines.

================================================================================
End of Glossary
================================================================================