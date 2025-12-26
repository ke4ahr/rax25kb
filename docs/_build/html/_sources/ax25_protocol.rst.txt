AX.25 Protocol
==============

Overview
--------

AX.25 (Amateur X.25) is a data link layer protocol derived from X.25 and specifically designed for amateur radio use. It provides reliable, connection-oriented or connectionless packet transmission over radio frequencies.

History
-------

AX.25 was developed in the 1980s based on the X.25 protocol stack, adapted for the unique characteristics of amateur radio:

* Half-duplex operation
* Variable propagation delays
* Shared channel access
* No guaranteed data rates

**Versions:**

* AX.25 v2.0 (1984) - Original specification
* AX.25 v2.2 (1997) - Current standard, adds improvements

Protocol Layer
--------------

AX.25 operates at the Data Link Layer (Layer 2) of the OSI model:

.. code-block:: text

   +------------------------+
   |    Application Layer   |  (APRS, NET/ROM, etc.)
   +------------------------+
   |    Network Layer       |  (NET/ROM, IP)
   +------------------------+
   |  AX.25 (Data Link)     |  ← We are here
   +------------------------+
   |    Physical Layer      |  (AFSK, FSK, PSK)
   +------------------------+

Frame Structure
---------------

Basic Frame Format
~~~~~~~~~~~~~~~~~~

An AX.25 frame consists of:

.. code-block:: text

   +------+------+------+-----+------+-----+-----+-----+
   | Flag | Dest | Src  | Digi| Ctrl | PID | Info| FCS |
   +------+------+------+-----+------+-----+-----+-----+
   
   Flag:  0x7E (not present in KISS)
   Dest:  Destination address (7 bytes)
   Src:   Source address (7 bytes)
   Digi:  Digipeater addresses (0-8, 7 bytes each)
   Ctrl:  Control field (1 or 2 bytes)
   PID:   Protocol ID (1 byte, optional)
   Info:  Information field (0-256 bytes)
   FCS:   Frame Check Sequence (2 bytes, not in KISS)

Address Field
~~~~~~~~~~~~~

Each address is 7 bytes:

.. code-block:: text

   Bytes 0-5: Callsign (6 characters, ASCII shifted left 1 bit)
   Byte 6:    SSID and flags
   
   SSID Byte Format:
   Bit 7 6 5 4 3 2 1 0
       C R R SSID  H/L
   
   C:    Command/Response bit
   R:    Reserved (set to 1)
   SSID: Station identifier (0-15)
   H/L:  Address extension bit (0=more addresses, 1=last)

**Example: KE4AHR-7**

.. code-block:: text

   Byte 0: 'K' << 1 = 0x96
   Byte 1: 'E' << 1 = 0x8A
   Byte 2: '4' << 1 = 0x68
   Byte 3: 'A' << 1 = 0x82
   Byte 4: 'H' << 1 = 0x90
   Byte 5: 'R' << 1 = 0xA4
   Byte 6: (7 << 1) | 0x61 = 0xEF (SSID=7, last address)

Control Field
~~~~~~~~~~~~~

The control field identifies the frame type:

**I-Frame (Information):**

.. code-block:: text

   Bit: 7 6 5 4 3 2 1 0
        N(R)     P N(S)  0
   
   N(S): Send sequence number
   N(R): Receive sequence number
   P:    Poll bit

**S-Frame (Supervisory):**

.. code-block:: text

   Bit: 7 6 5 4 3 2 1 0
        N(R)     P/F S S 0 1
   
   N(R): Receive sequence number
   P/F:  Poll/Final bit
   S S:  Supervisory function (00=RR, 01=RNR, 10=REJ, 11=SREJ)

**U-Frame (Unnumbered):**

.. code-block:: text

   Bit: 7 6 5 4 3 2 1 0
        M M M P/F M M 1 1
   
   M:    Mode bits (define command type)
   P/F:  Poll/Final bit

Protocol ID (PID)
~~~~~~~~~~~~~~~~~

Present in I-frames and UI-frames:

.. list-table::
   :header-rows: 1
   :widths: 20 80

   * - PID
     - Protocol
   * - 0x01
     - ISO 8208/CCITT X.25 PLP
   * - 0x06
     - Compressed TCP/IP
   * - 0x07
     - Uncompressed TCP/IP
   * - 0x08
     - Segmentation fragment
   * - 0xC3
     - TEXNET datagram
   * - 0xC4
     - Link Quality Protocol
   * - 0xCA
     - Appletalk
   * - 0xCB
     - Appletalk ARP
   * - 0xCC
     - ARPA Internet Protocol
   * - 0xCD
     - ARPA Address Resolution
   * - 0xCE
     - FlexNet
   * - 0xCF
     - NET/ROM
   * - 0xF0
     - No layer 3 (used by APRS)
   * - 0xFF
     - Escape character

Frame Types
-----------

I-Frames (Information)
~~~~~~~~~~~~~~~~~~~~~~

**Purpose:** Transfer data in connected mode

**Characteristics:**

* Contains sequence numbers
* Requires acknowledgment
* Supports flow control
* Error recovery via retransmission

**Example use:** File transfer, terminal connections

S-Frames (Supervisory)
~~~~~~~~~~~~~~~~~~~~~~

**Types:**

* **RR (Receive Ready)** - Ready to receive, acknowledges frames
* **RNR (Receive Not Ready)** - Busy, can't receive now
* **REJ (Reject)** - Request retransmission from N(R)
* **SREJ (Selective Reject)** - Request specific frame retransmission

**Purpose:** Flow control and error recovery in connected mode

U-Frames (Unnumbered)
~~~~~~~~~~~~~~~~~~~~~

**Common types:**

* **SABM (Set Asynchronous Balanced Mode)** - Establish connection
* **SABME** - SABM with extended sequence numbers
* **DISC (Disconnect)** - Close connection
* **DM (Disconnected Mode)** - Not connected response
* **UA (Unnumbered Acknowledgment)** - Acknowledge U-frame
* **FRMR (Frame Reject)** - Report protocol error
* **UI (Unnumbered Information)** - Connectionless data transfer

UI-Frames
~~~~~~~~~

**Special case:** Unnumbered Information frame

**Characteristics:**

* Connectionless (no SABM required)
* No acknowledgment
* No sequence numbers
* No retransmission
* Contains PID and information field

**Used by:** APRS, beacons, bulletins

Connection Phases
-----------------

Disconnected Phase
~~~~~~~~~~~~~~~~~~

Initial state, no connection established.

**Transitions:**

* Send SABM → Setup Phase
* Receive UI → Process connectionless data

Setup Phase
~~~~~~~~~~~

Establishing connection.

**Process:**

1. Station A sends SABM to Station B
2. Station B responds with UA
3. Connection established → Connected Phase

**Commands:**

* SABM - Request connection
* SABME - Request connection with extended mode
* UA - Accept connection
* DM - Reject connection

Connected Phase
~~~~~~~~~~~~~~~

Data transfer with reliability.

**Operations:**

* Send I-frames with data
* Receive acknowledgments (RR)
* Handle flow control (RNR)
* Retransmit on timeout or REJ

**Characteristics:**

* Sequence numbers track frames
* Acknowledgments ensure delivery
* Timeouts trigger retransmission

Disconnect Phase
~~~~~~~~~~~~~~~~

Closing connection.

**Process:**

1. Station sends DISC
2. Other station responds with UA
3. Return to Disconnected Phase

Error Recovery
~~~~~~~~~~~~~~

**Timeout:** Retransmit unacknowledged frames

**REJ:** Retransmit from specified sequence number

**FRMR:** Report protocol violation, disconnect

Digipeating
-----------

Digipeater Path
~~~~~~~~~~~~~~~

AX.25 supports up to 8 digipeater addresses:

.. code-block:: text

   Source → DIGI1 → DIGI2 → DIGI3 → Destination

**Address format:**

.. code-block:: text

   Src: KE4AHR-7
   Digi 1: WIDE1-1 (not repeated yet)
   Digi 2: WIDE2-1 (not repeated yet)
   Dest: APRS

H-Bit (Has-Been-Repeated)
~~~~~~~~~~~~~~~~~~~~~~~~~~

Bit 7 of SSID byte indicates if digipeater has repeated:

* 0 = Not repeated yet
* 1 = Already repeated

**Example progression:**

.. code-block:: text

   Original:  WIDE1-1 (H=0), WIDE2-1 (H=0)
   After #1:  WIDE1-1 (H=1), WIDE2-1 (H=0)
   After #2:  WIDE1-1 (H=1), WIDE2-1 (H=1)

Path Examples
~~~~~~~~~~~~~

**Direct:** No digipeaters

.. code-block:: text

   KE4AHR-7 → KE4AHR-1

**Single hop:**

.. code-block:: text

   KE4AHR-7 → WIDE1-1 → APRS

**Multi-hop:**

.. code-block:: text

   KE4AHR-7 → WIDE1-1,WIDE2-1 → APRS

**New paradigm:**

.. code-block:: text

   KE4AHR-7 → WIDE1-1,WIDE2-2 → APRS

APRS and AX.25
--------------

APRS uses AX.25 UI frames:

**Frame structure:**

.. code-block:: text

   Dest: APRS (or version-specific like APNU3F)
   Src: Station callsign with SSID
   Digi: Path (e.g., WIDE1-1,WIDE2-1)
   Ctrl: 0x03 (UI frame)
   PID: 0xF0 (no layer 3)
   Info: APRS data (position, weather, etc.)

**APRS data types:**

* Position reports (``!``, ``=``, ``/``, ``@``)
* Status (``>``)
* Messages (``:``)\
* Weather (``_``)
* Telemetry (``T``)
* Objects (``;``)

AX.25 in rax25kb
----------------

Frame Parsing
~~~~~~~~~~~~~

When ``-k`` is enabled, rax25kb parses AX.25 headers:

.. code-block:: text

   === TNC -> PC KISS Frame ===
     Port: 0, Command: 0 (Data)
     Frame length: 45 bytes
     AX.25: KE4AHR-7 > APRS
     Via: WIDE1-1, WIDE2-1
     Type: UIFrame
     Phase: UNCONNECTED (UI Frame)
     Control: 0x03
     PID: 0xf0
     Info: 29 bytes

Address Decoding
~~~~~~~~~~~~~~~~

rax25kb extracts callsigns and SSIDs:

.. code-block:: rust

   fn from_ax25_bytes(bytes: &[u8]) -> Option<AX25Address> {
       let mut callsign = String::new();
       for i in 0..6 {
           let ch = (bytes[i] >> 1) as char;
           if ch != ' ' { callsign.push(ch); }
       }
       let ssid = (bytes[6] >> 1) & 0x0F;
       Some(AX25Address { callsign, ssid })
   }

Frame Type Detection
~~~~~~~~~~~~~~~~~~~~

rax25kb identifies frame types:

.. code-block:: rust

   fn get_frame_type(&self) -> AX25FrameType {
       if (self.control & 0x01) == 0 { 
           IFrame  // Bit 0 = 0
       } else if (self.control & 0x03) == 0x01 { 
           SFrame  // Bits 1-0 = 01
       } else if (self.control & 0x03) == 0x03 {
           if (self.control & 0xEF) == 0x03 { 
               UIFrame  // UI specific
           } else { 
               UFrame 
           }
       } else { 
           Unknown 
       }
   }

Information Field Display
~~~~~~~~~~~~~~~~~~~~~~~~~

With ``-a``, rax25kb shows info field contents:

.. code-block:: text

   === TNC -> PC AX.25 Info Field (29 bytes) ===
   00000000: 21 34 32 30 34 2e 33 35  4e 2f 30 38 33 35 34 2e  !4204.35N/08354.
   00000010: 34 38 57 2d 50 48 47 20  32 31 35 30 2f           48W-PHG 2150/

Practical Examples
------------------

Beacon Frame
~~~~~~~~~~~~

**Raw AX.25:**

.. code-block:: text

   Dest:  APRS (0x82 0xA0 0xA6 0xA0 0x40 0x40 0xE0)
   Src:   KE4AHR-7 (0x96 0x8A 0x68 0x82 0x90 0xA4 0xEF)
   Ctrl:  0x03 (UI)
   PID:   0xF0 (No L3)
   Info:  !4204.35N/08354.48W-PHG 2150/

**KISS encapsulated:**

.. code-block:: text

   C0 00 82 A0 A6 A0 40 40 E0 96 8A 68 82 90 A4 EF
   03 F0 21 34 32 30 34 2E 33 35 4E 2F 30 38 33 35
   34 2E 34 38 57 2D 50 48 47 20 32 31 35 30 2F C0

Message Frame
~~~~~~~~~~~~~

**APRS message:**

.. code-block:: text

   :KE4AHR   :Test message{001

**Complete frame:**

.. code-block:: text

   Dest:  APRS
   Src:   KE4AHR-1
   Path:  WIDE1-1
   Ctrl:  0x03
   PID:   0xF0
   Info:  :KE4AHR   :Test message{001

Position with Timestamp
~~~~~~~~~~~~~~~~~~~~~~~

**APRS format:**

.. code-block:: text

   @221234z4204.35N/08354.48W>

**Fields:**

* ``@`` - Position with timestamp
* ``221234z`` - Day 22, time 12:34 UTC
* ``4204.35N/08354.48W`` - Coordinates
* ``>`` - Symbol (car)

Performance Considerations
--------------------------

Frame Size
~~~~~~~~~~

**Typical sizes:**

* Minimum: 16 bytes (header only)
* APRS position: 40-60 bytes
* APRS message: 50-80 bytes
* Maximum: ~330 bytes (with digipeaters and full info)

Throughput
~~~~~~~~~~

**1200 baud VHF:**

* Raw: 150 bytes/sec
* With overhead: ~100 bytes/sec effective
* Typical APRS beacon: 2-3 frames/sec max

**9600 baud:**

* Raw: 1200 bytes/sec
* With overhead: ~800 bytes/sec effective

Latency
~~~~~~~

**Components:**

* CSMA delay (persistence): 0-1000ms
* TX delay: 50-500ms
* Transmission time: (bytes * 8) / baud
* Propagation: negligible on VHF/UHF

**Example (60 byte frame @ 1200 baud):**

* Transmission: 400ms
* Total: 500-1500ms typical

References
----------

* AX.25 v2.2 Specification: https://www.tapr.org/pdf/AX25.2.2.pdf
* APRS Protocol Reference: http://www.aprs.org/doc/APRS101.PDF
* TAPR: https://www.tapr.org/
* Tucson Amateur Packet Radio: http://www.tapr.org/pr_intro.html

See Also
--------

* :doc:`kiss_protocol` - KISS protocol details
* :doc:`examples` - Practical examples
* :doc:`philflag` - Frame escaping corrections
