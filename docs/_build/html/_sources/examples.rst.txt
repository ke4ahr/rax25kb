Examples
========

This page provides practical examples of using rax25kb in various scenarios.

Basic Examples
--------------

Simple APRS Gateway
~~~~~~~~~~~~~~~~~~~

**Linux:**

.. code-block:: bash

   # Start with minimal configuration
   rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001

**Windows:**

.. code-block:: powershell

   # Start with minimal configuration
   rax25kb.exe -D COM3 -b 9600 -p 8001

**What it does:**

* Connects to serial port at 9600 baud
* Listens on port 8001 for TCP connections
* Provides transparent KISS bridge

Development and Testing
~~~~~~~~~~~~~~~~~~~~~~~

**Enable full diagnostics:**

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -b 9600 \
           -k -a -d \
           -L 7 \
           -l /tmp/rax25kb-debug.log

**Output shows:**

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
   
   === TNC -> PC AX.25 Info Field (29 bytes) ===
   00000000: 21 34 32 30 34 2e 33 35  4e 2f 30 38 33 35 34 2e  !4204.35N/08354.
   00000010: 34 38 57 2d 50 48 47 20  32 31 35 30 2f           48W-PHG 2150/

Packet Capture for Wireshark
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -k --pcap /tmp/packets.pcap

**Analyzing with Wireshark:**

.. code-block:: bash

   wireshark /tmp/packets.pcap

**Using tcpdump:**

.. code-block:: bash

   tcpdump -r /tmp/packets.pcap -X

Windows-Specific Examples
--------------------------

Basic Windows Setup
~~~~~~~~~~~~~~~~~~~

**Using Command Prompt:**

.. code-block:: batch

   REM Navigate to installation directory
   cd "C:\Program Files\rax25kb"
   
   REM Run with COM3 port
   rax25kb.exe -D COM3 -b 9600 -p 8001

**Using PowerShell:**

.. code-block:: powershell

   # Navigate to installation directory
   Set-Location "C:\Program Files\rax25kb"
   
   # Run with COM3 port
   .\rax25kb.exe -D COM3 -b 9600 -p 8001

**Output:**

.. code-block:: text

   rax25kb - AX.25 KISS Bridge
   ==========================
   Configuration from: rax25kb.cfg
     Serial: COM3 @ 9600 baud
     Flow control: None
     Format: 8N1 (8 data bits, None parity, 1 stop bit)
     TCP: 0.0.0.0 port 8001
     PhilFlag: OFF
     Raw copy: OFF
     Log level: 5
   
   [2025-12-20 10:30:45] [NOTICE] KISS-over-TCP server listening on 0.0.0.0:8001

Windows with Configuration File
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Create config file** ``C:\Program Files\rax25kb\rax25kb.cfg``:

.. code-block:: ini

   # Windows configuration
   serial_port=COM3
   baud_rate=9600
   stop_bits=1
   parity=none
   flow_control=none
   
   tcp_address=127.0.0.1
   tcp_port=8001
   
   log_level=5
   logfile=C:\ProgramData\rax25kb\rax25kb.log
   pidfile=C:\ProgramData\rax25kb\rax25kb.pid

**Run with config file:**

.. code-block:: powershell

   .\rax25kb.exe -c "C:\Program Files\rax25kb\rax25kb.cfg"

Windows Service Setup (NSSM)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Download and install NSSM:**

1. Download from https://nssm.cc/download
2. Extract to ``C:\Program Files\nssm``

**Install service:**

.. code-block:: powershell

   # Run as Administrator
   cd "C:\Program Files\nssm\win64"
   
   # Install service
   .\nssm.exe install rax25kb "C:\Program Files\rax25kb\rax25kb.exe"
   
   # Set parameters
   .\nssm.exe set rax25kb AppParameters -D COM3 -b 9600 -p 8001
   .\nssm.exe set rax25kb AppDirectory "C:\Program Files\rax25kb"
   .\nssm.exe set rax25kb DisplayName "rax25kb KISS Bridge"
   .\nssm.exe set rax25kb Description "AX.25 KISS protocol bridge"
   .\nssm.exe set rax25kb Start SERVICE_AUTO_START
   
   # Set logging
   .\nssm.exe set rax25kb AppStdout "C:\ProgramData\rax25kb\service.log"
   .\nssm.exe set rax25kb AppStderr "C:\ProgramData\rax25kb\error.log"
   
   # Start service
   .\nssm.exe start rax25kb

**Manage service:**

.. code-block:: powershell

   # Check status
   .\nssm.exe status rax25kb
   
   # Stop service
   .\nssm.exe stop rax25kb
   
   # Restart service
   .\nssm.exe restart rax25kb
   
   # Remove service
   .\nssm.exe remove rax25kb confirm

Windows Task Scheduler Setup
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Create batch file** ``C:\Program Files\rax25kb\start-rax25kb.bat``:

.. code-block:: batch

   @echo off
   cd "C:\Program Files\rax25kb"
   rax25kb.exe -D COM3 -b 9600 -p 8001 -l "C:\ProgramData\rax25kb\rax25kb.log"

**Create scheduled task:**

1. Open Task Scheduler
2. Create Task → General tab:
   - Name: rax25kb
   - Run whether user is logged on or not
   - Run with highest privileges
3. Triggers tab → New:
   - Begin: At startup
4. Actions tab → New:
   - Action: Start a program
   - Program: ``C:\Program Files\rax25kb\start-rax25kb.bat``
5. Conditions tab:
   - Uncheck "Start only if on AC power"
6. Settings tab:
   - Allow task to run on demand
   - If running task does not end when requested, force it to stop

Windows Firewall Configuration
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Allow through Windows Firewall:**

.. code-block:: powershell

   # Run as Administrator
   New-NetFirewallRule -DisplayName "rax25kb" `
                       -Direction Inbound `
                       -Program "C:\Program Files\rax25kb\rax25kb.exe" `
                       -Action Allow `
                       -Protocol TCP `
                       -LocalPort 8001

**Using GUI:**

1. Open Windows Defender Firewall
2. Advanced Settings → Inbound Rules → New Rule
3. Program → Browse to ``rax25kb.exe``
4. Allow the connection
5. Apply to Domain, Private, and Public
6. Name: "rax25kb KISS Bridge"

Windows with APRS Software
~~~~~~~~~~~~~~~~~~~~~~~~~~~

**APRSIS32 Configuration:**

1. Start rax25kb:

   .. code-block:: powershell

      .\rax25kb.exe -D COM3 -b 9600 -p 8001

2. Configure APRSIS32:
   - Configure → Ports
   - Add Port → Type: KISS/Network
   - TCP Address: ``localhost:8001``
   - Enable port

**UI-View32 Configuration:**

1. Setup → Comms Setup → KISS
2. Host: ``localhost``
3. Port: ``8001``
4. Connect

Advanced Examples
-----------------

High-Speed TNC with Flow Control
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Linux:**

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -b 115200 -H -k -p 8001

**Windows:**

.. code-block:: powershell

   rax25kb.exe -D COM3 -b 115200 -H -k -p 8001

**Configuration file equivalent:**

.. code-block:: ini

   serial_port=/dev/ttyUSB0
   baud_rate=115200
   flow_control=hardware
   tcp_port=8001
   parse_kiss=1

Multiple TNCs on Different Ports
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**TNC 1 (port 8001):**

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001 \
           -P /var/run/rax25kb-tnc1.pid \
           -l /var/log/rax25kb-tnc1.log

**TNC 2 (port 8002):**

.. code-block:: bash

   rax25kb -D /dev/ttyUSB1 -b 9600 -p 8002 \
           -P /var/run/rax25kb-tnc2.pid \
           -l /var/log/rax25kb-tnc2.log

IPv6 Only Configuration
~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -I "::" -6 -p 8001

Dual Stack (IPv4 + IPv6)
~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -I "0.0.0.0 ::" -p 8001

TASCO Modem with PhilFlag
~~~~~~~~~~~~~~~~~~~~~~~~~~

**Full diagnostic mode:**

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -b 9600 \
           -n -k -a \
           -L 7 \
           -l /var/log/rax25kb-tasco.log \
           --pcap /var/log/tasco-packets.pcap

**Windows:**

.. code-block:: powershell

   rax25kb.exe -D COM3 -b 9600 `
               -n -k -a `
               -L 7 `
               -l "C:\ProgramData\rax25kb\tasco.log" `
               --pcap "C:\ProgramData\rax25kb\packets.pcap"

Raw Mode for TNC Configuration
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Access TNC command mode:**

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -b 9600 -R -p 8001

**Connect with telnet:**

.. code-block:: bash

   telnet localhost 8001

**Send TNC commands:**

.. code-block:: text

   INT KISS
   TXDELAY 30
   RESTART

**Exit raw mode:**

Press ``Ctrl+]`` then type ``quit``

Integration Examples
--------------------

Direwolf Integration
~~~~~~~~~~~~~~~~~~~~

**Start rax25kb:**

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001

**Configure Direwolf** (``direwolf.conf``):

.. code-block:: ini

   ADEVICE null null
   KISSPORT 8001
   AGWPORT 8000
   
   CHANNEL 0
   MYCALL KE4AHR-7
   MODEM 1200
   
   PBEACON delay=1 every=30 overlay=S symbol="digi" \
           lat=42^04.35N long=083^54.48W comment="Direwolf iGate"

**Start Direwolf:**

.. code-block:: bash

   direwolf -c direwolf.conf -t 0

Xastir Integration
~~~~~~~~~~~~~~~~~~

**Linux:**

1. Start rax25kb:

   .. code-block:: bash

      rax25kb -D /dev/ttyUSB0 -b 9600 -k -p 8001

2. Configure Xastir:
   - Interface → Add → Network KISS
   - Host: ``localhost``
   - Port: ``8001``
   - Device name: ``rax25kb``
   - Connect

APRX Integration
~~~~~~~~~~~~~~~~

**Configure APRX** (``/etc/aprx.conf``):

.. code-block:: text

   mycall KE4AHR-7
   
   <interface>
      serial-device /dev/tnc0 ttyUSB0 9600 8n1 KISS
      callsign KE4AHR-7
      tx-ok true
   </interface>
   
   <beacon>
      beaconmode both
      cycle-size 30m
      beacon symbol "R&" lat "4204.35N" lon "08354.48W" \
             comment "APRX Digi"
   </beacon>

**Start services:**

.. code-block:: bash

   # Start rax25kb
   rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001
   
   # Start APRX
   aprx -d

Systemd Service with Logging
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Create service file** ``/etc/systemd/system/rax25kb.service``:

.. code-block:: ini

   [Unit]
   Description=rax25kb AX.25 KISS Bridge
   After=network.target
   
   [Service]
   Type=simple
   User=radio
   Group=dialout
   WorkingDirectory=/home/radio
   
   ExecStartPre=/bin/mkdir -p /var/log/rax25kb
   ExecStartPre=/bin/chown radio:dialout /var/log/rax25kb
   
   ExecStart=/usr/local/bin/rax25kb \
             -D /dev/ttyUSB0 \
             -b 9600 \
             -k \
             -p 8001 \
             -l /var/log/rax25kb/rax25kb.log \
             -P /var/run/rax25kb.pid \
             --pcap /var/log/rax25kb/packets.pcap
   
   Restart=on-failure
   RestartSec=5
   
   [Install]
   WantedBy=multi-user.target

**Enable and manage:**

.. code-block:: bash

   sudo systemctl daemon-reload
   sudo systemctl enable rax25kb
   sudo systemctl start rax25kb
   sudo systemctl status rax25kb
   
   # View logs
   sudo journalctl -u rax25kb -f
   
   # Rotate log file
   sudo logrotate /etc/logrotate.d/rax25kb

Docker Container Example
~~~~~~~~~~~~~~~~~~~~~~~~

**Create Dockerfile:**

.. code-block:: dockerfile

   FROM rust:latest AS builder
   
   WORKDIR /build
   COPY . .
   RUN cargo build --release
   
   FROM debian:bookworm-slim
   
   RUN apt-get update && \
       apt-get install -y --no-install-recommends \
       ca-certificates && \
       rm -rf /var/lib/apt/lists/*
   
   COPY --from=builder /build/target/release/rax25kb /usr/local/bin/
   COPY examples/rax25kb.cfg /etc/rax25kb/rax25kb.cfg
   
   EXPOSE 8001
   
   CMD ["rax25kb", "-c", "/etc/rax25kb/rax25kb.cfg"]

**Build and run:**

.. code-block:: bash

   docker build -t rax25kb .
   
   docker run -d \
              --name rax25kb \
              --device=/dev/ttyUSB0:/dev/ttyUSB0 \
              -p 8001:8001 \
              -v /var/log/rax25kb:/var/log/rax25kb \
              rax25kb

Troubleshooting Examples
-------------------------

Test Serial Port Access
~~~~~~~~~~~~~~~~~~~~~~~

**Linux:**

.. code-block:: bash

   # Check if port exists
   ls -l /dev/ttyUSB0
   
   # Test with minimal config
   rax25kb -D /dev/ttyUSB0 -b 9600 -q
   
   # If permission denied
   sudo usermod -a -G dialout $USER
   # Log out and back in

**Windows:**

.. code-block:: powershell

   # List COM ports
   Get-WmiObject Win32_SerialPort | Select-Object Name,DeviceID
   
   # Test with minimal config
   .\rax25kb.exe -D COM3 -b 9600 -q

Verify TCP Connection
~~~~~~~~~~~~~~~~~~~~~

**Start server:**

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001

**Test with telnet:**

.. code-block:: bash

   telnet localhost 8001

**Test with netcat:**

.. code-block:: bash

   nc localhost 8001

**Windows PowerShell:**

.. code-block:: powershell

   Test-NetConnection -ComputerName localhost -Port 8001

Debug Packet Flow
~~~~~~~~~~~~~~~~~

**Full verbosity:**

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -b 9600 \
           -k -a -d \
           -L 9 \
           -l /tmp/debug.log

**Monitor in real-time:**

.. code-block:: bash

   tail -f /tmp/debug.log

**Windows:**

.. code-block:: powershell

   Get-Content "C:\ProgramData\rax25kb\debug.log" -Wait

Performance Testing
-------------------

Throughput Test
~~~~~~~~~~~~~~~

**Setup:**

.. code-block:: bash

   # Terminal 1: Start rax25kb
   rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001 -L 3
   
   # Terminal 2: Send test data
   yes "TEST DATA PACKET" | head -n 1000 | nc localhost 8001

**Monitor:**

.. code-block:: bash

   # Check connection
   netstat -an | grep 8001
   
   # Monitor CPU usage
   top -p $(cat /var/run/rax25kb.pid)

Stress Test
~~~~~~~~~~~

**Multiple simultaneous connections:**

.. code-block:: bash

   for i in {1..10}; do
       (echo "Connection $i" | nc localhost 8001) &
   done

Quick Reference
---------------

Common Command Patterns
~~~~~~~~~~~~~~~~~~~~~~~

**Basic startup:**

.. code-block:: bash

   rax25kb -D DEVICE -b BAUD -p PORT

**With diagnostics:**

.. code-block:: bash

   rax25kb -D DEVICE -k -a -L 7

**Production mode:**

.. code-block:: bash

   rax25kb -c CONFIG -P PIDFILE -l LOGFILE -q

**TASCO modem:**

.. code-block:: bash

   rax25kb -D DEVICE -n -k

**Raw mode:**

.. code-block:: bash

   rax25kb -D DEVICE -R

Port and Baud Rate Combinations
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # 1200 baud (VHF packet)
   rax25kb -D /dev/ttyUSB0 -b 1200 -p 8001
   
   # 9600 baud (standard)
   rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001
   
   # 19200 baud (high speed)
   rax25kb -D /dev/ttyUSB0 -b 19200 -p 8001
   
   # 115200 baud (USB TNC)
   rax25kb -D /dev/ttyUSB0 -b 115200 -p 8001

See Also
--------

* :doc:`usage` - Detailed usage information
* :doc:`configuration` - Configuration reference
* :doc:`troubleshooting` - Problem solving
* :doc:`philflag` - PhilFlag documentation
