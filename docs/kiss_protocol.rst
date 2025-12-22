KISS Protocol
=============

Overview
--------

KISS (Keep It Simple, Stupid) is a protocol for communicating with Terminal Node Controllers (TNCs) used in amateur radio packet communication. It provides a simple, transparent method for passing data between a computer and a TNC.

History
-------

The KISS protocol was developed by Mike Chepponis (K3MC) and Phil Karn (KA9Q) in 1986 as a simpler alternative to the complex command-based TNC interfaces of the time. The goal was to move packet protocol processing from the TNC firmware to the host computer software.

**Key advantages:**

* Simplicity - minimal protocol overhead
* Transparency - raw AX.25 frames pass through
* Flexibility - host software controls all packet handling
* Multi-port support - up to 16 ports per TNC

Protocol Specification
----------------------

Special Characters
~~~~~~~~~~~~~~~~~~

KISS uses four special bytes for frame delimiting and escaping:

.. list-table::
   :header-rows: 1
   :widths: 20 15 15 50

   * - Name
     - Hex
     - Decimal
     - Purpose
   * - FEND
     - 0xC0
     - 192
     - Frame End/Start delimiter
   * - FESC
     - 0xDB
     - 219
     - Frame Escape
   * - TFEND
     - 0xDC
     - 220
     - Transposed Frame End
   * - TFESC
     - 0xDD
     - 221
     - Transposed Frame Escape

Frame Structure
~~~~~~~~~~~~~~~

A KISS frame has the following structure:

.. code-block:: text

   +------+----------+------+
   | FEND | CMD+DATA | FEND |
   +------+----------+------+
   
   Where:
   - FEND (0xC0) marks frame boundaries
   - CMD is the command byte
   - DATA is the payload (optional)

Command Byte Format
~~~~~~~~~~~~~~~~~~~

The command byte combines port number and command:

.. code-block:: text

   Bit:  7  6  5  4  3  2  1  0
        +--+--+--+--+--+--+--+--+
        | Port  |   Command     |
        +--+--+--+--+--+--+--+--+
   
   Port:    Bits 7-4 (0-15)
   Command: Bits 3-0 (0-15)

KISS Commands
~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 15 15 70

   * - Command
     - Value
     - Description
   * - Data Frame
     - 0x00
     - AX.25 data frame to/from TNC
   * - TX Delay
     - 0x01
     - Transmit delay (10ms units)
   * - Persistence
     - 0x02
     - Persistence parameter (p-persistence)
   * - Slot Time
     - 0x03
     - Slot time (10ms units)
   * - TX Tail
     - 0x04
     - Time to keep transmitter on after packet
   * - Full Duplex
     - 0x05
     - 0=half duplex, 1=full duplex
   * - Set Hardware
     - 0x06
     - Hardware-specific commands
   * - Return
     - 0x0F
     - Exit KISS mode (return to command mode)

Escaping Rules
~~~~~~~~~~~~~~

When FEND (0xC0) or FESC (0xDB) appear in the data, they must be escaped:

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Byte in Data
     - KISS Encoding
   * - 0xC0 (FEND)
     - 0xDB 0xDC (FESC TFEND)
   * - 0xDB (FESC)
     - 0xDB 0xDD (FESC TFESC)

**Example:**

.. code-block:: text

   Data byte:       ... 0xC0 0x42 0xDB ...
   KISS encoded:    ... 0xDB 0xDC 0x42 0xDB 0xDD ...

Frame Examples
--------------

Data Frame (Port 0)
~~~~~~~~~~~~~~~~~~~

**Structure:**

.. code-block:: text

   C0 00 [AX.25 frame data] C0
   
   Where:
   - C0 = FEND (start)
   - 00 = Port 0, Command 0 (Data Frame)
   - [AX.25 frame data] = escaped payload
   - C0 = FEND (end)

**Real example:**

.. code-block:: text

   C0 00 9E 82 A0 A6 40 40 E0 9A 88 34 82 90 A4 61
   03 F0 3D 34 32 30 34 2E 33 35 4E 2F 30 38 33 35
   34 2E 34 38 57 2D C0

**Breakdown:**

* ``C0`` - Frame start
* ``00`` - Port 0, Data command
* ``9E 82 A0 A6 40 40 E0`` - Destination address (APRS)
* ``9A 88 34 82 90 A4 61`` - Source address (KE4AHR-7)
* ``03`` - Control byte (UI frame)
* ``F0`` - PID (no layer 3)
* ``3D 34 32...`` - Information field (=4204.35N/08354.48W-)
* ``C0`` - Frame end

Set TX Delay
~~~~~~~~~~~~

.. code-block:: text

   C0 01 1E C0
   
   Sets TX delay to 30 * 10ms = 300ms

Set Persistence
~~~~~~~~~~~~~~~

.. code-block:: text

   C0 02 3F C0
   
   Sets persistence to 63/256 = ~25%

Exit KISS Mode
~~~~~~~~~~~~~~

.. code-block:: text

   C0 0F C0
   
   Returns TNC to command mode

Data Frame with Escaping
~~~~~~~~~~~~~~~~~~~~~~~~~

**Unescaped data:**

.. code-block:: text

   Data: C0 00 [... payload with 0xC0 byte ...] C0

**Problem:** The 0xC0 in payload looks like frame delimiter

**Properly escaped:**

.. code-block:: text

   C0 00 [... payload ...] DB DC [... more payload ...] C0
                          ^^^^^^^
                          Escaped 0xC0

Implementation in rax25kb
-------------------------

Frame Reception
~~~~~~~~~~~~~~~

rax25kb receives KISS frames from the serial port:

.. code-block:: rust

   // Pseudo-code
   while reading from serial {
       if byte == FEND {
           if frame_buffer has data {
               process_frame(frame_buffer)
               clear frame_buffer
           }
           start new frame
       } else {
           append byte to frame_buffer
       }
   }

Frame Parsing
~~~~~~~~~~~~~

When parse_kiss is enabled (``-k``), rax25kb decodes frames:

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

Escape Sequence Handling
~~~~~~~~~~~~~~~~~~~~~~~~~

rax25kb properly handles escape sequences:

.. code-block:: rust

   // Decoding
   if byte == FESC {
       next_byte = read()
       if next_byte == TFEND {
           output(FEND)  // 0xC0
       } else if next_byte == TFESC {
           output(FESC)  // 0xDB
       }
   }
   
   // Encoding
   if byte == FEND {
       output(FESC, TFEND)  // 0xDB 0xDC
   } else if byte == FESC {
       output(FESC, TFESC)  // 0xDB 0xDD
   }

Multi-Port Support
~~~~~~~~~~~~~~~~~~

KISS supports up to 16 ports (0-15):

.. code-block:: text

   Port 0:  C0 00 [data] C0  (command byte = 0x00)
   Port 1:  C0 10 [data] C0  (command byte = 0x10)
   Port 2:  C0 20 [data] C0  (command byte = 0x20)
   ...
   Port 15: C0 F0 [data] C0  (command byte = 0xF0)

Common Issues
-------------

Missing Escape Sequences
~~~~~~~~~~~~~~~~~~~~~~~~

**Problem:** TNC doesn't escape 0xC0 in data

**Symptoms:**

* Corrupted frames
* Unexpected frame boundaries
* Packet loss

**Solution:** Use PhilFlag (``-n``) - see :doc:`philflag`

Incorrect Frame Delimiters
~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Problem:** Extra FEND bytes

Some TNCs send multiple FEND bytes:

.. code-block:: text

   C0 C0 00 [data] C0 C0

This is acceptable and should be handled gracefully.

Timing Issues
~~~~~~~~~~~~~

**TX Delay too short:**

* First packet bytes lost
* Incomplete frames

**TX Delay too long:**

* Channel access delays
* Reduced throughput

**Typical values:**

* HF: 300-500ms
* VHF/UHF: 200-300ms
* Fast TNCs: 50-100ms

Persistence Algorithm
~~~~~~~~~~~~~~~~~~~~~

The p-persistence CSMA algorithm:

1. Sense channel (carrier detect)
2. If busy, wait until clear
3. Generate random number (0-255)
4. If random < persistence value, transmit
5. Otherwise wait one slot time and repeat

**Example:**

.. code-block:: text

   Persistence = 63 (0x3F) = ~25% chance per slot
   Slot time = 10 (100ms)

Protocol Limitations
--------------------

No Error Detection
~~~~~~~~~~~~~~~~~~

KISS provides no checksums or error detection. The AX.25 FCS (Frame Check Sequence) must be calculated by the TNC hardware.

No Flow Control
~~~~~~~~~~~~~~~

KISS has no built-in flow control. Buffer overruns are possible on high-speed TNCs. Use hardware flow control (RTS/CTS) when available.

No Acknowledgments
~~~~~~~~~~~~~~~~~~

KISS provides no acknowledgment mechanism. Data is sent without confirmation of receipt.

Single Direction
~~~~~~~~~~~~~~~~

Each frame is independent. No session management or connection state.

KISS vs Other TNC Modes
-----------------------

Command Mode
~~~~~~~~~~~~

**Traditional TNC command mode:**

* Human-readable commands (e.g., ``MYCALL KE4AHR``)
* Complex configuration
* Protocol processing in TNC firmware

**KISS mode:**

* Binary protocol
* Minimal TNC involvement
* Protocol processing in host software

KISS vs SMACK
~~~~~~~~~~~~~

SMACK (State Machine for Appended CRC's and KISS) adds:

* CRC checking
* Connection state
* Acknowledgments

KISS vs 6PACK
~~~~~~~~~~~~~

6PACK protocol adds:

* Multiple virtual TNCs over one serial port
* Priority levels
* Better error handling

KISS is simpler and more widely supported.

Testing and Debugging
----------------------

Frame Capture
~~~~~~~~~~~~~

Enable hex dump to see raw KISS frames:

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -d

Output:

.. code-block:: text

   === TNC -> PC KISS Frame (45 bytes) ===
   00000000: c0 00 9e 82 a0 a6 40 40  e0 9a 88 34 82 90 a4 61  ......@@...4...a
   00000010: 03 f0 3d 34 32 30 34 2e  33 35 4e 2f 30 38 33 35  ..=4204.35N/0835
   00000020: 34 2e 34 38 57 2d c0                              4.48W-.

Parse KISS Frames
~~~~~~~~~~~~~~~~~

Enable KISS parsing:

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -k

Manual Frame Construction
~~~~~~~~~~~~~~~~~~~~~~~~~

**Send test frame with netcat:**

.. code-block:: bash

   # Start rax25kb
   rax25kb -D /dev/ttyUSB0 -p 8001
   
   # Create test frame (hex)
   echo -ne '\xC0\x00TEST\xC0' | nc localhost 8001

**Python example:**

.. code-block:: python

   import socket
   
   # KISS frame: FEND + CMD + DATA + FEND
   frame = b'\xC0\x00' + b'TEST DATA' + b'\xC0'
   
   sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
   sock.connect(('localhost', 8001))
   sock.send(frame)
   sock.close()

References
----------

Specifications
~~~~~~~~~~~~~~

* TAPR KISS Specification: http://www.ax25.net/kiss.aspx
* Original KISS paper: http://www.ka9q.net/papers/kiss.html
* AX.25 v2.2 Specification: https://www.tapr.org/pdf/AX25.2.2.pdf

Related Standards
~~~~~~~~~~~~~~~~~

* AX.25 Link Layer Protocol
* APRS Protocol Specification
* TCP/IP over AX.25

Further Reading
~~~~~~~~~~~~~~~

* "Packet Radio Networking" - Chepponis & Karn
* ARRL Digital Communications Handbook
* Linux AX.25 HOWTO

See Also
--------

* :doc:`ax25_protocol` - AX.25 protocol details
* :doc:`philflag` - KISS escaping corrections
* :doc:`examples` - Practical examples
* :doc:`troubleshooting` - Common problems
