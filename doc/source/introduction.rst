Introduction
============

What is rax25kb?
----------------

**rax25kb** is a sophisticated multi-port AX.25 KISS bridge designed for amateur radio packet communications. It bridges serial KISS/Extended KISS (XKISS) TNCs to TCP networks and enables serial-to-serial connections with protocol translation.

Key Features
------------

Multi-Port Support
~~~~~~~~~~~~~~~~~~

* Support for up to 10,000 independent cross-connects (cross_connect0000 to cross_connect9999)
* Each cross-connect operates independently with its own configuration
* Mix serial-to-TCP and serial-to-serial bridges in one instance

Protocol Translation
~~~~~~~~~~~~~~~~~~~~

* **KISS Support**: Standard KISS protocol with port numbers 0-15
* **Extended KISS (XKISS)**: Advanced port addressing beyond standard limitations
* **Bidirectional Translation**: Automatic KISS ↔ XKISS conversion
* **Port Mapping**: Map KISS port 0 to any XKISS port number

PhilFlag Correction
~~~~~~~~~~~~~~~~~~~

Addresses bugs in TASCO modem chipsets:

* **Serial → TCP**: Escapes embedded 0xC0 bytes (FEND) within frames
* **TCP → Serial**: Escapes C/c characters to prevent TC0\\n command parsing

Advanced Features
~~~~~~~~~~~~~~~~~

* **PCAP Capture**: Record AX.25 frames to standard PCAP format
* **Frame Parsing**: Display KISS and AX.25 frame information
* **Raw Copy Mode**: Transparent byte-for-byte pass-through
* **Flexible Logging**: Multi-level logging to console and/or file
* **Single Connection**: One TCP client per cross-connect for reliability

Use Cases
---------

Traditional KISS Bridge
~~~~~~~~~~~~~~~~~~~~~~~

Connect a KISS TNC to the network for remote access:

.. code-block:: text

    [KISS TNC] ← Serial → [rax25kb] ← TCP → [Application]

Multi-Port Gateway
~~~~~~~~~~~~~~~~~~

Bridge multiple TNCs to different TCP ports:

.. code-block:: text

    [TNC 1] ← Serial → [rax25kb] ← TCP:8001
    [TNC 2] ← Serial → [rax25kb] ← TCP:8002
    [TNC 3] ← Serial → [rax25kb] ← TCP:8003

KISS to XKISS Translation
~~~~~~~~~~~~~~~~~~~~~~~~~~

Interconnect standard KISS and Extended KISS devices:

.. code-block:: text

    [KISS TNC] ← Serial → [rax25kb] ← Serial → [XKISS TNC]
         (Port 0)     (Translation)              (Port 5)

Serial Port Multiplexer
~~~~~~~~~~~~~~~~~~~~~~~

Connect multiple TNCs together with protocol translation:

.. code-block:: text

    [TNC A] ← Serial → [rax25kb] ← Serial → [TNC B]
    [TNC C] ← Serial → [rax25kb] ← Serial → [TNC D]

Why rax25kb?
------------

Reliability
~~~~~~~~~~~

* Single connection per port prevents contention
* Thread-safe operation with proper locking
* Graceful error handling and recovery
* Stable long-term operation

Flexibility
~~~~~~~~~~~

* Configuration file or command-line operation
* Per-bridge feature configuration
* Mix and match bridge types
* Extensive logging and debugging options

Performance
~~~~~~~~~~~

* Independent threads per bridge
* Minimal latency
* Efficient buffer management
* Optimized for continuous operation

Compatibility
~~~~~~~~~~~~~

* **Operating Systems**: Linux, Windows, macOS
* **TNCs**: Any KISS or XKISS compatible device
* **Applications**: Works with AGWPE, Direwolf, soundmodem, and others
* **Protocols**: AX.25, compatible with APRS, packet radio, etc.

History
-------

**rax25kb** was created to address specific issues with TASCO modem chipsets that improperly handled KISS framing. Over time, it evolved into a comprehensive multi-port bridge with advanced features for modern amateur radio packet operations.

Version 1.6.5 introduces the cross-connect architecture, enabling multiple independent bridges in a single instance with KISS/XKISS translation capabilities.

License
-------

**rax25kb** is free software licensed under the GNU General Public License v3.0 or later (GPL-3.0-or-later).

You are free to:

* Use the software for any purpose
* Study and modify the source code
* Distribute copies
* Distribute modified versions

See the LICENSE file for full terms.

Getting Started
---------------

1. **Install**: Follow the :doc:`installation` guide for your platform
2. **Configure**: Create a configuration file (see :doc:`configuration`)
3. **Run**: Start rax25kb with your configuration
4. **Connect**: Point your application to the TCP port

For detailed instructions, see:

* :doc:`installation` - Installation instructions
* :doc:`configuration` - Configuration guide
* :doc:`usage` - Usage examples
* :doc:`windows` - Windows-specific instructions

Support
-------

* **Documentation**: https://github.com/ke4ahr/rax25kb/
* **Issues**: https://github.com/ke4ahr/rax25kb/issues
* **Source Code**: https://github.com/ke4ahr/rax25kb/

Community
---------

**rax25kb** is an open-source project. Contributions are welcome!

See :doc:`contributing` for guidelines on submitting issues, feature requests, and code contributions.
