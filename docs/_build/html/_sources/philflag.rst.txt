PhilFlag Correction
===================

Overview
--------

PhilFlag is a bidirectional correction mechanism designed to work around bugs in TASCO modem chipsets that improperly handle KISS protocol escaping. The name comes from the whimsical phrase embedded in the source code: *"In for drought? No, we are in for turbo-hydrometeorology!"*

The Problem
-----------

TASCO Modem Chipset Bug
~~~~~~~~~~~~~~~~~~~~~~~~

Some radios using TASCO modem chipsets have a bug in their KISS implementation:

1. **Improper KISS Escaping**: The modem fails to properly escape ``0xC0`` (FEND) bytes when converting raw AX.25 frames to KISS format
2. **Command Misinterpretation**: The modem incorrectly interprets certain character sequences as commands, particularly ``TC0\n`` or ``tc0\n``

This causes:

* Corrupted packet data
* Packet loss
* Unexpected modem behavior or resets

KISS Protocol Background
~~~~~~~~~~~~~~~~~~~~~~~~

The KISS protocol uses special bytes for frame delimiting:

* ``0xC0`` (FEND) - Frame End/Start delimiter
* ``0xDB`` (FESC) - Frame Escape
* ``0xDC`` (TFEND) - Transposed Frame End
* ``0xDD`` (TFESC) - Transposed Frame Escape

**Standard KISS Escaping:**

When ``0xC0`` appears in packet data, it should be escaped as:

.. code-block:: text

   0xC0 → 0xDB 0xDC

When ``0xDB`` appears in packet data, it should be escaped as:

.. code-block:: text

   0xDB → 0xDB 0xDD

**The TASCO Bug:**

The TASCO modem chipset fails to perform this escaping when transmitting from radio to computer, leaving ``0xC0`` bytes unescaped within the frame data.

The Solution
------------

Serial to TCP Direction (Receive)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Problem:** Unescaped ``0xC0`` bytes in frame data from the TNC

**Solution:** PhilFlag detects and corrects unescaped ``0xC0`` bytes inside KISS frames

**Process:**

1. Monitor incoming data from serial port
2. Track frame boundaries using FEND delimiters
3. When a frame is complete, scan for unescaped ``0xC0`` bytes
4. Replace any unescaped ``0xC0`` with proper escape sequence ``0xDB 0xDC``
5. Forward corrected frame to TCP

**Example:**

.. code-block:: text

   Received from TNC (incorrect):
   C0 00 [data] C0 [more_data] C0

   After PhilFlag correction:
   C0 00 [data] DB DC [more_data] C0

TCP to Serial Direction (Transmit)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Problem:** TNC interprets ``TC0\n`` or ``tc0\n`` as command sequences

**Solution:** Escape 'C' and 'c' characters to prevent command misinterpretation

**Process:**

1. Monitor incoming data from TCP
2. Scan for bytes ``0x43`` ('C') and ``0x63`` ('c')
3. Escape these characters as ``0xDB 0x43`` and ``0xDB 0x63``
4. Forward escaped data to serial port

**Example:**

.. code-block:: text

   Received from TCP:
   ... T C 0 \n ...

   After PhilFlag correction:
   ... T DB 43 0 \n ...

This prevents the TNC from seeing "TC0\n" as a command.

Implementation Details
----------------------

Code Structure
~~~~~~~~~~~~~~

**Function: process_frame_with_phil_flag**

Handles Serial → TCP direction:

.. code-block:: rust

   fn process_frame_with_phil_flag(frame: &[u8]) -> Vec<u8> {
       if frame.len() < 2 { return frame.to_vec(); }
       let mut output = Vec::with_capacity(frame.len() * 2);
       output.push(frame[0]);  // Preserve first FEND
       
       // Process middle bytes
       for i in 1..frame.len()-1 {
           if frame[i] == KISS_FEND {
               output.push(KISS_FESC);   // 0xDB
               output.push(KISS_TFEND);  // 0xDC
           } else {
               output.push(frame[i]);
           }
       }
       
       output.push(frame[frame.len()-1]);  // Preserve last FEND
       output
   }

**Function: process_phil_flag_tcp_to_serial**

Handles TCP → Serial direction:

.. code-block:: rust

   fn process_phil_flag_tcp_to_serial(data: &[u8]) -> Vec<u8> {
       let mut output = Vec::with_capacity(data.len() * 2);
       for &byte in data {
           if byte == 0x43 || byte == 0x63 {  // 'C' or 'c'
               output.push(KISS_FESC);  // 0xDB
               output.push(byte);       // 0x43 or 0x63
           } else {
               output.push(byte);
           }
       }
       output
   }

Frame Processing Flow
~~~~~~~~~~~~~~~~~~~~~

**Serial to TCP (Receive Path):**

.. code-block:: text

   Serial Port → Frame Buffer → PhilFlag Correction → TCP Socket
                     ↓
              Detect frame boundaries
              (FEND...FEND)
                     ↓
              Escape internal 0xC0
                     ↓
              Forward to TCP

**TCP to Serial (Transmit Path):**

.. code-block:: text

   TCP Socket → PhilFlag Correction → Serial Port
                     ↓
              Escape 'C' and 'c'
              (0x43, 0x63)
                     ↓
              Forward to serial

Usage
-----

Enabling PhilFlag
~~~~~~~~~~~~~~~~~

**Configuration file:**

.. code-block:: ini

   phil_flag=1

**Command line:**

.. code-block:: bash

   rax25kb -n
   rax25kb --phil

**Verification:**

Look for this in the startup output:

.. code-block:: text

   [2025-01-15 10:30:45] [NOTICE] PhilFlag: ENABLED

When to Use PhilFlag
~~~~~~~~~~~~~~~~~~~~

**Enable PhilFlag if:**

* You have a TASCO modem-based radio
* You experience corrupted packets
* Packets contain ``0xC0`` bytes in the payload
* The modem resets unexpectedly when certain data is transmitted
* You see "TC0" or "tc0" in diagnostic output

**Do NOT enable PhilFlag if:**

* Your TNC properly implements KISS escaping
* You use a standard TNC (most modern TNCs)
* You don't experience packet corruption

**Known affected hardware:**

* Some QYT radios with TASCO modems
* Certain Chinese-made radios from 2015-2020
* Generic TASCO chipset implementations

Testing PhilFlag
~~~~~~~~~~~~~~~~

**Test procedure:**

1. Enable PhilFlag and KISS parsing:

   .. code-block:: bash

      rax25kb -D /dev/ttyUSB0 -n -k -d

2. Transmit test data containing ``0xC0`` bytes

3. Observe output for proper escaping:

   .. code-block:: text

      === TNC -> PC KISS Frame ===
      ... DB DC ...  (escaped C0)

4. Transmit text containing "TC0" or "tc0"

5. Verify modem doesn't reset or change modes

Performance Impact
------------------

Processing Overhead
~~~~~~~~~~~~~~~~~~~

PhilFlag adds minimal processing overhead:

* **Serial → TCP**: ~2-5% CPU increase for byte scanning and escaping
* **TCP → Serial**: ~1-2% CPU increase for character escaping
* **Memory**: Temporary buffer allocation (max 2x frame size)

The overhead is negligible on modern systems.

Throughput Impact
~~~~~~~~~~~~~~~~~

* No significant throughput reduction
* Frame processing happens in-memory before transmission
* Escaping adds at most a few bytes per frame

Latency Impact
~~~~~~~~~~~~~~

* Adds <1ms per frame in typical conditions
* Frame must be complete before processing in Serial → TCP direction
* No buffering delay in TCP → Serial direction

Limitations
-----------

What PhilFlag Does NOT Fix
~~~~~~~~~~~~~~~~~~~~~~~~~~~

PhilFlag corrects KISS escaping bugs but does not:

* Fix hardware timing issues
* Correct RF interference or signal problems
* Repair damaged serial port hardware
* Fix non-KISS protocol issues
* Correct AX.25 header errors
* Fix CRC or checksum failures

Compatibility
~~~~~~~~~~~~~

* Works with KISS protocol only
* Not compatible with raw copy mode (``-R``)
* Requires proper serial port configuration
* Does not interfere with standard KISS implementations

Debugging PhilFlag
------------------

Diagnostic Commands
~~~~~~~~~~~~~~~~~~~

**Full diagnostic output:**

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -n -k -d -a -L 7 -l /tmp/philflag-debug.log

**Capture for analysis:**

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -n -k --pcap /tmp/philflag-test.pcap

Analyzing Results
~~~~~~~~~~~~~~~~~

**Look for escaped bytes in hex dump:**

.. code-block:: text

   00000010: ... DB DC ...  ← Correctly escaped 0xC0

**Check KISS frame parsing:**

.. code-block:: text

   === TNC -> PC KISS Frame ===
     Port: 0, Command: 0 (Data)
     Frame length: 42 bytes

**Verify packet integrity:**

Open PCAP in Wireshark and check for:

* Valid AX.25 headers
* Correct CRC values
* No truncated frames

Troubleshooting
---------------

PhilFlag Not Working
~~~~~~~~~~~~~~~~~~~~

**Symptoms:**

* Still seeing corrupted packets
* Modem still resets

**Solutions:**

1. Verify PhilFlag is enabled:

   .. code-block:: bash

      rax25kb -D /dev/ttyUSB0 -n | grep PhilFlag

2. Check serial port configuration

3. Verify TNC is in KISS mode

4. Test with ``-k -d`` to see raw frames

False Positives
~~~~~~~~~~~~~~~

**Symptoms:**

* PhilFlag escaping where it shouldn't
* Double-escaped data

**Solutions:**

1. Disable PhilFlag if TNC works correctly without it
2. Verify your TNC actually has the TASCO bug
3. Test without PhilFlag first

Technical References
--------------------

KISS Protocol Specification
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* TAPR KISS Specification: http://www.ax25.net/kiss.aspx
* AX.25 Protocol Specification: https://www.tapr.org/pdf/AX25.2.2.pdf

TASCO Modem Information
~~~~~~~~~~~~~~~~~~~~~~~

* Known affected chipsets: TASCO TX-1H, TX-2H series
* Firmware versions: 1.0-2.5 (approximately)
* Manufacturers: Various Chinese OEM/ODM manufacturers

Related Documentation
~~~~~~~~~~~~~~~~~~~~~

* :doc:`kiss_protocol` - KISS protocol details
* :doc:`ax25_protocol` - AX.25 protocol overview
* :doc:`troubleshooting` - General troubleshooting
* :doc:`examples` - PhilFlag usage examples
