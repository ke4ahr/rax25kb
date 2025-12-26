Frequently Asked Questions
==========================

General Questions
-----------------

What is rax25kb?
~~~~~~~~~~~~~~~~

rax25kb is a bridge application that connects serial port KISS TNCs to TCP/IP networks. It allows APRS and other AX.25 software to communicate with hardware TNCs over TCP instead of direct serial connections.

Why would I use rax25kb?
~~~~~~~~~~~~~~~~~~~~~~~~~

**Use cases:**

* Run APRS software on a different computer than the TNC
* Share one TNC with multiple applications
* Access TNCs remotely over network
* Work around serial port limitations
* Test and debug packet radio applications
* Integrate with modern software that expects network connections

What does "KISS" mean?
~~~~~~~~~~~~~~~~~~~~~~

KISS stands for "Keep It Simple, Stupid" - a protocol designed to simplify communication between computers and TNCs by moving packet processing from the TNC to the host computer.

Is rax25kb free?
~~~~~~~~~~~~~~~~

Yes, rax25kb is free and open source software licensed under GPL-3.0-or-later.

Installation and Setup
----------------------

What operating systems are supported?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

rax25kb runs on:

* Linux (all distributions)
* Windows (7 and later)
* macOS (10.12 and later)
* FreeBSD, OpenBSD, and other Unix-like systems

Do I need to install Rust?
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Only if you're building from source. Pre-compiled binaries may be available for your platform.

How do I find my serial port name?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Linux:**

.. code-block:: bash

   ls /dev/ttyUSB* /dev/ttyACM*
   dmesg | grep tty

**Windows:**

.. code-block:: powershell

   Get-WmiObject Win32_SerialPort | Select-Object Name,DeviceID

Or check Device Manager → Ports (COM & LPT)

**macOS:**

.. code-block:: bash

   ls /dev/cu.*
   ls /dev/tty.*

Why do I get "Permission denied" on Linux?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Your user needs to be in the ``dialout`` or ``uucp`` group:

.. code-block:: bash

   sudo usermod -a -G dialout $USER

Log out and back in for changes to take effect.

Configuration
-------------

What baud rate should I use?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Common baud rates:**

* 1200 - VHF/UHF packet radio (AFSK)
* 9600 - High-speed packet (FSK)
* 19200 - Faster USB TNCs
* 115200 - Modern USB devices

Most KISS TNCs use 9600 or 115200 baud for the serial connection.

Should I use flow control?
~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Guidelines:**

* **None** - Works for most TNCs (default)
* **Hardware (RTS/CTS)** - Recommended for high-speed TNCs
* **Software (XON/XOFF)** - Older equipment, rarely needed
* **DTR/DSR** - Specific hardware requirements

When in doubt, start with no flow control.

What's the correct serial format?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Standard KISS TNCs use **8N1**:

* 8 data bits
* No parity
* 1 stop bit

This is rax25kb's default setting.

What TCP port should I use?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Common choices:**

* 8001 - Traditional KISS-over-TCP port
* 10001 - Alternative
* Any unused port 1024-65535

Choose a port not used by other services.

Should I bind to 0.0.0.0 or 127.0.0.1?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**0.0.0.0** - Listen on all network interfaces (accessible remotely)

**127.0.0.1** - Listen on localhost only (local access only)

For security, use 127.0.0.1 unless remote access is needed.

Usage
-----

How do I test if rax25kb is working?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

1. Start rax25kb:

   .. code-block:: bash

      rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001

2. Connect with telnet:

   .. code-block:: bash

      telnet localhost 8001

3. If you see a connection established, it's working!

Can multiple applications connect simultaneously?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

No, rax25kb accepts one TCP connection at a time per listener. For multiple clients, you need multiple instances on different ports or use a multiplexer.

How do I see what packets are being received?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Enable KISS parsing:

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -k

Or enable full diagnostics:

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -k -a -L 7

Why aren't packets showing up in my APRS software?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Check:**

1. TNC is in KISS mode (not command mode)
2. Serial port settings are correct
3. TCP connection is established
4. APRS software is configured for KISS over TCP
5. TNC is receiving packets (check with ``-k``)

How do I put my TNC in KISS mode?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Most TNCs:**

1. Connect to TNC in command mode
2. Send command: ``KISS ON`` or ``INT KISS``
3. TNC enters KISS mode

**To exit KISS mode:**

rax25kb can send the exit command:

.. code-block:: text

   C0 0F C0

Or use raw mode:

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -R -p 8001
   telnet localhost 8001
   # Type: INT KISS OFF

PhilFlag
--------

What is PhilFlag?
~~~~~~~~~~~~~~~~~

PhilFlag is a correction mechanism for buggy TASCO modem chipsets that don't properly escape KISS protocol bytes. See :doc:`philflag` for details.

When should I enable PhilFlag?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Enable if:**

* You have a TASCO modem-based radio
* Packets are corrupted
* Modem resets unexpectedly

**Don't enable if:**

* Your TNC works fine without it
* You have a modern, standards-compliant TNC

How do I know if I need PhilFlag?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Test without PhilFlag first. If you experience corrupted packets or unexpected behavior, try enabling it:

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -n -k

Does PhilFlag slow down performance?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

No, the overhead is minimal (<5% CPU increase).

Windows-Specific
----------------

Why can't Windows find my COM port?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Check:**

1. Device Manager → Ports (COM & LPT)
2. Verify COM port number (COM1, COM3, etc.)
3. Ensure driver is installed
4. Try unplugging and replugging USB device

How do I run rax25kb as a Windows service?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Use NSSM (Non-Sucking Service Manager):

.. code-block:: powershell

   nssm install rax25kb "C:\Program Files\rax25kb\rax25kb.exe"
   nssm set rax25kb AppParameters -D COM3 -b 9600 -p 8001
   nssm start rax25kb

See :doc:`examples` for detailed instructions.

Can I use rax25kb with APRSIS32?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Yes! Configure APRSIS32:

1. Configure → Ports
2. Add Port → Type: KISS/Network
3. TCP Address: ``localhost:8001``
4. Enable port

How do I configure Windows Firewall?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: powershell

   New-NetFirewallRule -DisplayName "rax25kb" `
                       -Direction Inbound `
                       -Protocol TCP `
                       -LocalPort 8001 `
                       -Action Allow

Troubleshooting
---------------

Why do I get "Failed to open serial port"?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Possible causes:**

1. Wrong port name
2. Insufficient permissions
3. Port already in use
4. Device not connected
5. Driver not installed

**Solutions:**

* Verify port exists: ``ls /dev/ttyUSB*`` (Linux)
* Check permissions: ``ls -l /dev/ttyUSB0`` (Linux)
* Close other programs using the port
* Check USB connection

Why does the connection keep dropping?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Possible causes:**

1. Serial cable issue
2. USB power problem
3. RF interference
4. Baud rate mismatch
5. Flow control mismatch

**Solutions:**

* Check physical connections
* Try different USB port
* Use shielded cables
* Verify baud rate matches TNC
* Try different flow control settings

How do I enable debug logging?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -L 7 -l /tmp/debug.log

Or maximum verbosity:

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -L 9 -l /tmp/debug.log

Why are my packets corrupted?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Possible causes:**

1. TASCO modem bug (needs PhilFlag)
2. Serial port errors
3. Baud rate mismatch
4. RF interference
5. TNC firmware bug

**Solutions:**

* Try PhilFlag: ``-n``
* Verify serial settings match TNC
* Check RF environment
* Update TNC firmware

How do I capture packets for analysis?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Use PCAP capture:

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 --pcap /tmp/packets.pcap

Analyze with Wireshark:

.. code-block:: bash

   wireshark /tmp/packets.pcap

Performance
-----------

How much CPU does rax25kb use?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Very little - typically <1% on modern systems. With PhilFlag and full parsing, maybe 2-5%.

Can rax25kb handle high-speed data?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Yes, rax25kb is designed for efficiency and can handle:

* 9600 baud with ease
* 115200 baud on most systems
* Multiple simultaneous frames

What's the maximum throughput?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Limited by serial baud rate and protocol overhead:

* 1200 baud: ~150 bytes/sec
* 9600 baud: ~1200 bytes/sec
* 115200 baud: ~14,400 bytes/sec

Integration
-----------

Can I use rax25kb with Direwolf?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Yes! Direwolf can connect to rax25kb as a KISS TCP client.

Does rax25kb work with APRS-IS?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

No, rax25kb bridges TNCs to TCP. For APRS-IS, use an iGate application that connects to both the TNC (via rax25kb) and APRS-IS.

Can I chain multiple rax25kb instances?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

No direct chaining, but you can run multiple instances for multiple TNCs on different ports.

Does rax25kb support multi-port TNCs?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Yes, KISS multi-port is supported. Ports 0-15 are handled according to the KISS specification.

Development
-----------

How do I build from source?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   git clone https://github.com/ke4ahr/rax25kb.git
   cd rax25kb
   cargo build --release

Is there an API?
~~~~~~~~~~~~~~~~

No, rax25kb is a bridge application. It provides KISS-over-TCP as its interface.

Can I contribute?
~~~~~~~~~~~~~~~~~

Yes! See :doc:`contributing` for guidelines.

Where do I report bugs?
~~~~~~~~~~~~~~~~~~~~~~~~

Report bugs on the GitHub repository issue tracker.

Support
-------

Where can I get help?
~~~~~~~~~~~~~~~~~~~~~

* Documentation: https://www.outpostpm.org/support.html
* GitHub Issues: https://github.com/ke4ahr/rax25kb/issues
* Amateur radio forums
* Local amateur radio club

Is there commercial support?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

No, rax25kb is community-supported open source software.

How do I contact the developers?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Through the GitHub repository or amateur radio channels.

Still Have Questions?
---------------------

Check these resources:

* :doc:`troubleshooting` - Detailed problem solving
* :doc:`examples` - Practical examples
* :doc:`usage` - Usage documentation
* GitHub Issues - Search existing issues or create new ones
