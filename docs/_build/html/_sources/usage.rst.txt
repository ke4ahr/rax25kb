Usage
=====

Basic Usage
-----------

Starting rax25kb
~~~~~~~~~~~~~~~~

With default configuration file:

.. code-block:: bash

   rax25kb

With custom configuration file:

.. code-block:: bash

   rax25kb -c /etc/rax25kb/myconfig.cfg

Stopping rax25kb
~~~~~~~~~~~~~~~~

Press ``Ctrl+C`` to stop the application gracefully:

.. code-block:: text

   ^C
   Received SIGINT, shutting down gracefully...

Command-Line Options
--------------------

Help and Version
~~~~~~~~~~~~~~~~

Display help message:

.. code-block:: bash

   rax25kb --help
   rax25kb -h

Serial Port Options
~~~~~~~~~~~~~~~~~~~

**Specify serial device:**

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0        # Linux
   rax25kb -D COM3                # Windows

**Set baud rate:**

.. code-block:: bash

   rax25kb -b 9600
   rax25kb -b 115200

**Configure stop bits:**

.. code-block:: bash

   rax25kb -s 1    # 1 stop bit (default for KISS)
   rax25kb -s 2    # 2 stop bits

**Set parity:**

.. code-block:: bash

   rax25kb -Q none   # No parity (default for KISS)
   rax25kb -Q even   # Even parity
   rax25kb -Q odd    # Odd parity

Flow Control Options
~~~~~~~~~~~~~~~~~~~~

**No flow control (default):**

.. code-block:: bash

   rax25kb -N
   rax25kb --none

**Software flow control (XON/XOFF):**

.. code-block:: bash

   rax25kb -x
   rax25kb --xon-xoff

**Hardware flow control (RTS/CTS):**

.. code-block:: bash

   rax25kb -H
   rax25kb --rts-cts

**DTR/DSR flow control:**

.. code-block:: bash

   rax25kb --dtr-dsr

Network Options
~~~~~~~~~~~~~~~

**Specify TCP address:**

.. code-block:: bash

   rax25kb -I 127.0.0.1            # Localhost only
   rax25kb -I 0.0.0.0              # All IPv4 interfaces
   rax25kb -I "0.0.0.0 ::"         # IPv4 + IPv6

**Set TCP port:**

.. code-block:: bash

   rax25kb -p 8001
   rax25kb -p 10001

**Force IPv4 or IPv6:**

.. code-block:: bash

   rax25kb -I "0.0.0.0 ::" -4      # IPv4 only
   rax25kb -I "0.0.0.0 ::" -6      # IPv6 only

Feature Options
~~~~~~~~~~~~~~~

**Enable frame dumping:**

.. code-block:: bash

   rax25kb -d
   rax25kb --dump

**Enable KISS parsing:**

.. code-block:: bash

   rax25kb -k
   rax25kb --kiss

**Enable AX.25 info field dumping:**

.. code-block:: bash

   rax25kb -a
   rax25kb --ax25

**Enable PhilFlag correction:**

.. code-block:: bash

   rax25kb -n
   rax25kb --phil

**Enable raw copy mode:**

.. code-block:: bash

   rax25kb -R
   rax25kb --raw-copy

Logging Options
~~~~~~~~~~~~~~~

**Set log file:**

.. code-block:: bash

   rax25kb -l /var/log/rax25kb.log
   rax25kb --logfile /var/log/rax25kb.log

**Set log level (0-9):**

.. code-block:: bash

   rax25kb -L 5      # NOTICE (default)
   rax25kb -L 7      # DEBUG
   rax25kb -L 9      # VERBOSE

**Console-only logging:**

.. code-block:: bash

   rax25kb --console-only

**File-only logging:**

.. code-block:: bash

   rax25kb --no-console -l /var/log/rax25kb.log

Output Options
~~~~~~~~~~~~~~

**Write PID file:**

.. code-block:: bash

   rax25kb -P /var/run/rax25kb.pid
   rax25kb --pidfile /var/run/rax25kb.pid

**Capture packets to PCAP:**

.. code-block:: bash

   rax25kb --pcap /var/log/packets.pcap

**Quiet startup:**

.. code-block:: bash

   rax25kb -q
   rax25kb --quiet

Common Usage Patterns
---------------------

Basic APRS Gateway
~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -b 9600 -I 0.0.0.0 -p 8001

With KISS Parsing
~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -b 9600 -k

Debug Mode with Full Logging
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -k -a -L 7 -l /var/log/rax25kb-debug.log

PhilFlag Correction for TASCO Modems
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -n -k

Packet Capture for Analysis
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -k --pcap /tmp/packets.pcap

High-Speed TNC with Hardware Flow Control
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -b 115200 -H -k

Raw Mode for Direct TNC Access
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -R

Production Deployment
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -c /etc/rax25kb/production.cfg \
           -P /var/run/rax25kb.pid \
           -l /var/log/rax25kb.log \
           -L 5 \
           -q

Understanding Output
--------------------

Startup Information
~~~~~~~~~~~~~~~~~~~

Normal startup displays:

.. code-block:: text

   rax25kb - AX.25 KISS Bridge
   ==========================
   Configuration from: rax25kb.cfg
     Serial: /dev/ttyUSB0 @ 9600 baud
     Flow control: None
     Format: 8N1 (8 data bits, None parity, 1 stop bit)
     TCP: 0.0.0.0 port 8001
     PhilFlag: OFF
     Raw copy: OFF
     Log level: 5
   
   [2025-01-15 10:30:45] [NOTICE] rax25kb starting
   [2025-01-15 10:30:45] [NOTICE] KISS-over-TCP server listening on 0.0.0.0:8001
   [2025-01-15 10:30:45] [NOTICE] PhilFlag: DISABLED

Log Messages
~~~~~~~~~~~~

Log format:

.. code-block:: text

   [YYYY-MM-DD HH:MM:SS] [LEVEL] message

Example log messages:

.. code-block:: text

   [2025-01-15 10:31:20] [NOTICE] Client connected: 192.168.1.100:54321
   [2025-01-15 10:31:25] [INFO] Received KISS frame: 42 bytes
   [2025-01-15 10:31:30] [NOTICE] Client disconnected
   [2025-01-15 10:31:30] [NOTICE] Connection closed

KISS Frame Display
~~~~~~~~~~~~~~~~~~

With ``-k`` option:

.. code-block:: text

   === TNC -> PC KISS Frame ===
     Port: 0, Command: 0 (Data)
     Frame length: 42 bytes
     AX.25: KE4AHR > APRS
     Type: UIFrame
     Phase: UNCONNECTED (UI Frame)
     Control: 0x03
     PID: 0xf0
     Info: 28 bytes

AX.25 Info Field Display
~~~~~~~~~~~~~~~~~~~~~~~~~

With ``-k -a`` options:

.. code-block:: text

   === TNC -> PC AX.25 Info Field (28 bytes) ===
   00000000: 3d 34 32 30 34 2e 33 35  4e 2f 30 38 33 35 34 2e  =4204.35N/08354.
   00000010: 34 38 57 2d 50 48 47 20  32 2e 31 30             48W-PHG 2.10

Hex Dump Display
~~~~~~~~~~~~~~~~

With ``-d`` option:

.. code-block:: text

   === TNC -> PC KISS Frame (42 bytes) ===
   00000000: c0 00 9e 82 a0 a6 40 40  e0 9a 88 34 82 90 a4 61  ......@@...4...a
   00000010: 03 f0 3d 34 32 30 34 2e  33 35 4e 2f 30 38 33 35  ..=4204.35N/0835
   00000020: 34 2e 34 38 57 2d c0                              4.48W-.

Connecting to rax25kb
---------------------

Using Telnet
~~~~~~~~~~~~

.. code-block:: bash

   telnet localhost 8001

Using netcat
~~~~~~~~~~~~

.. code-block:: bash

   nc localhost 8001

Using APRS Software
~~~~~~~~~~~~~~~~~~~

Configure your APRS software to connect to:

* **Host:** localhost (or IP address)
* **Port:** 8001 (or configured port)
* **Protocol:** KISS over TCP

Example (Xastir):

1. Interface → Add → Network KISS
2. Host: localhost
3. Port: 8001
4. Connect

Example (APRSIS32):

1. Configure → Ports
2. Add Port → Type: KISS/Network
3. TCP Address: localhost:8001

Running as a Service
--------------------

Linux systemd Service
~~~~~~~~~~~~~~~~~~~~~

Create service file ``/etc/systemd/system/rax25kb.service``:

.. code-block:: ini

   [Unit]
   Description=rax25kb AX.25 KISS Bridge
   After=network.target
   
   [Service]
   Type=simple
   User=radio
   Group=dialout
   WorkingDirectory=/home/radio
   ExecStart=/usr/local/bin/rax25kb -c /etc/rax25kb/rax25kb.cfg
   Restart=on-failure
   RestartSec=5
   
   [Install]
   WantedBy=multi-user.target

Enable and start:

.. code-block:: bash

   sudo systemctl daemon-reload
   sudo systemctl enable rax25kb
   sudo systemctl start rax25kb

Check status:

.. code-block:: bash

   sudo systemctl status rax25kb

View logs:

.. code-block:: bash

   sudo journalctl -u rax25kb -f

Windows Service
~~~~~~~~~~~~~~~

Using NSSM (Non-Sucking Service Manager):

1. Download NSSM from https://nssm.cc/
2. Install service:

   .. code-block:: powershell

      nssm install rax25kb "C:\Program Files\rax25kb\rax25kb.exe"
      nssm set rax25kb AppParameters -c "C:\Program Files\rax25kb\rax25kb.cfg"
      nssm set rax25kb AppDirectory "C:\Program Files\rax25kb"

3. Start service:

   .. code-block:: powershell

      nssm start rax25kb

Monitoring and Management
--------------------------

Check Process Status
~~~~~~~~~~~~~~~~~~~~

Using PID file:

.. code-block:: bash

   ps -p $(cat /var/run/rax25kb.pid)

Using ps:

.. code-block:: bash

   ps aux | grep rax25kb

Monitor Log File
~~~~~~~~~~~~~~~~

.. code-block:: bash

   tail -f /var/log/rax25kb.log

Check Network Connections
~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   netstat -tlnp | grep 8001
   ss -tlnp | grep 8001

Analyze PCAP Capture
~~~~~~~~~~~~~~~~~~~~~

Using Wireshark:

.. code-block:: bash

   wireshark /var/log/rax25kb.pcap

Using tcpdump:

.. code-block:: bash

   tcpdump -r /var/log/rax25kb.pcap -A

Next Steps
----------

* :doc:`examples` - See more practical examples
* :doc:`troubleshooting` - Solve common problems
* :doc:`faq` - Find answers to common questions
