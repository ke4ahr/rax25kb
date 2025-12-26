Troubleshooting
===============

This guide helps diagnose and resolve common issues with rax25kb.

Serial Port Issues
------------------

Cannot Open Serial Port
~~~~~~~~~~~~~~~~~~~~~~~~

**Error:**

.. code-block:: text

   Failed to open serial port '/dev/ttyUSB0': Permission denied

**Cause:** Insufficient permissions

**Solution (Linux):**

.. code-block:: bash

   # Add user to dialout group
   sudo usermod -a -G dialout $USER
   
   # Log out and back in, then verify
   groups

**Alternative (temporary):**

.. code-block:: bash

   sudo chmod 666 /dev/ttyUSB0

**Solution (Windows):**

Run Command Prompt or PowerShell as Administrator

Serial Port Not Found
~~~~~~~~~~~~~~~~~~~~~~

**Error:**

.. code-block:: text

   Failed to open serial port '/dev/ttyUSB0': No such file or directory

**Linux diagnostics:**

.. code-block:: bash

   # List all serial devices
   ls /dev/ttyUSB* /dev/ttyACM* /dev/ttyS*
   
   # Check USB devices
   lsusb
   
   # Check kernel messages
   dmesg | grep tty
   dmesg | grep usb

**Windows diagnostics:**

.. code-block:: powershell

   # List COM ports
   Get-WmiObject Win32_SerialPort | Select-Object Name,DeviceID
   
   # Or use Device Manager
   devmgmt.msc

**Common causes:**

* Device not plugged in
* USB cable faulty
* Driver not installed
* Wrong port name

Port Already in Use
~~~~~~~~~~~~~~~~~~~

**Error:**

.. code-block:: text

   Failed to open serial port: Device or resource busy

**Find what's using the port:**

.. code-block:: bash

   # Linux
   lsof | grep ttyUSB0
   fuser /dev/ttyUSB0
   
   # Check for other rax25kb instances
   ps aux | grep rax25kb

**Solution:**

.. code-block:: bash

   # Kill the process using the port
   sudo fuser -k /dev/ttyUSB0
   
   # Or identify and close the application

TCP/Network Issues
------------------

Cannot Bind to TCP Port
~~~~~~~~~~~~~~~~~~~~~~~~

**Error:**

.. code-block:: text

   Failed to bind to 0.0.0.0:8001: Address already in use

**Find what's using the port:**

.. code-block:: bash

   # Linux
   sudo netstat -tlnp | grep 8001
   sudo ss -tlnp | grep 8001
   sudo lsof -i :8001
   
   # macOS
   sudo lsof -i :8001
   
   # Windows
   netstat -ano | findstr :8001

**Solution:**

1. Use a different port: ``rax25kb -p 8002``
2. Stop the conflicting service
3. Kill the process: ``sudo kill <PID>``

Connection Refused
~~~~~~~~~~~~~~~~~~

**Symptoms:**

.. code-block:: bash

   telnet localhost 8001
   # Connection refused

**Diagnostics:**

.. code-block:: bash

   # Check if rax25kb is running
   ps aux | grep rax25kb
   
   # Check if port is listening
   netstat -tln | grep 8001
   
   # Check firewall
   sudo iptables -L | grep 8001

**Solutions:**

1. Verify rax25kb is running
2. Check bind address (127.0.0.1 vs 0.0.0.0)
3. Check firewall rules
4. Verify correct port number

Cannot Connect Remotely
~~~~~~~~~~~~~~~~~~~~~~~~

**Problem:** Local connections work, remote connections fail

**Causes:**

1. Bound to 127.0.0.1 instead of 0.0.0.0
2. Firewall blocking connections
3. Network routing issues

**Solutions:**

**Bind to all interfaces:**

.. code-block:: bash

   rax25kb -I 0.0.0.0 -p 8001

**Linux firewall:**

.. code-block:: bash

   sudo iptables -A INPUT -p tcp --dport 8001 -j ACCEPT
   sudo ufw allow 8001/tcp

**Windows firewall:**

.. code-block:: powershell

   New-NetFirewallRule -DisplayName "rax25kb" `
                       -Direction Inbound `
                       -Protocol TCP `
                       -LocalPort 8001 `
                       -Action Allow

**Test locally first:**

.. code-block:: bash

   telnet localhost 8001

**Then test remotely:**

.. code-block:: bash

   telnet <server-ip> 8001

Connection Drops
~~~~~~~~~~~~~~~~

**Symptoms:**

* Connection randomly disconnects
* "Connection reset by peer" errors

**Causes:**

1. Network issues
2. TCP timeout
3. Application crash
4. Serial port errors

**Diagnostics:**

.. code-block:: bash

   # Monitor logs
   rax25kb -L 7 -l /tmp/rax25kb.log
   tail -f /tmp/rax25kb.log
   
   # Check system logs
   sudo journalctl -u rax25kb -f
   dmesg | tail

**Solutions:**

* Check network connectivity
* Verify serial cable connection
* Check USB power
* Monitor for system errors

Data/Packet Issues
------------------

No Packets Received
~~~~~~~~~~~~~~~~~~~

**Symptoms:**

* rax25kb starts successfully
* No packets appear in logs
* APRS software shows no activity

**Diagnostics:**

.. code-block:: bash

   # Enable full diagnostics
   rax25kb -D /dev/ttyUSB0 -k -a -d -L 7
   
   # Check if TNC is in KISS mode
   rax25kb -D /dev/ttyUSB0 -R -p 8001
   telnet localhost 8001
   # Type: CMD: (should get prompt if NOT in KISS mode)

**Solutions:**

1. **Put TNC in KISS mode:**

   .. code-block:: text

      INT KISS
      # or
      KISS ON

2. **Verify serial settings match TNC**
3. **Check antenna connection**
4. **Verify frequency settings**
5. **Ensure TNC is receiving RF**

Corrupted Packets
~~~~~~~~~~~~~~~~~

**Symptoms:**

* Garbled text in APRS software
* Invalid callsigns
* CRC errors

**Diagnostics:**

.. code-block:: bash

   # Capture packets
   rax25kb -D /dev/ttyUSB0 -k -a --pcap /tmp/packets.pcap
   
   # Analyze with Wireshark
   wireshark /tmp/packets.pcap

**Common causes:**

1. **TASCO modem bug** - Try PhilFlag:

   .. code-block:: bash

      rax25kb -D /dev/ttyUSB0 -n -k

2. **Baud rate mismatch:**

   .. code-block:: bash

      # Try different baud rates
      rax25kb -D /dev/ttyUSB0 -b 9600
      rax25kb -D /dev/ttyUSB0 -b 115200

3. **Serial port errors:**

   .. code-block:: bash

      # Check for errors
      dmesg | grep ttyUSB
      # Look for "frame error", "overrun", etc.

4. **RF interference:**

   * Check antenna/feedline
   * Verify frequency
   * Check for nearby transmitters

Partial Packets
~~~~~~~~~~~~~~~

**Symptoms:**

* Packets cut off mid-stream
* Missing data at end of frames

**Possible causes:**

1. Buffer overflow
2. TX delay too short
3. Flow control needed

**Solutions:**

.. code-block:: bash

   # Enable hardware flow control
   rax25kb -D /dev/ttyUSB0 -H
   
   # Increase TX delay in TNC
   # (requires raw mode or TNC configuration)

Duplicate Packets
~~~~~~~~~~~~~~~~~

**Symptoms:**

* Same packet appears multiple times
* APRS shows duplicate positions

**Causes:**

* Normal digipeater operation (expected)
* Software bug
* Multiple TNCs receiving same packet

**Verification:**

.. code-block:: bash

   rax25kb -k  # Check packet headers
   # Look for different digipeater paths

TNC Issues
----------

TNC Not in KISS Mode
~~~~~~~~~~~~~~~~~~~~~

**Symptoms:**

* No packets received
* Strange characters in output
* TNC responds to commands

**Solution:**

.. code-block:: bash

   # Enter raw mode
   rax25kb -D /dev/ttyUSB0 -R -p 8001
   
   # Connect with telnet
   telnet localhost 8001
   
   # Send KISS command
   INT KISS
   # or
   KISS ON
   
   # Exit telnet (Ctrl+] then quit)
   # Restart rax25kb in normal mode

TNC Not Responding
~~~~~~~~~~~~~~~~~~

**Diagnostics:**

.. code-block:: bash

   # Test serial port
   screen /dev/ttyUSB0 9600
   # or
   minicom -D /dev/ttyUSB0 -b 9600

**Send test commands:**

.. code-block:: text

   CMD:
   INT KISS OFF
   MYCALL TEST
   CMD:

**If no response:**

1. Check baud rate
2. Verify serial cable
3. Check TNC power
4. Try hardware reset

Cannot Exit KISS Mode
~~~~~~~~~~~~~~~~~~~~~~

**Problem:** TNC stuck in KISS mode

**Solution 1: Send KISS return command:**

.. code-block:: bash

   # Using raw mode
   rax25kb -D /dev/ttyUSB0 -R -p 8001
   
   # Connect and send
   echo -ne '\xC0\x0F\xC0' | nc localhost 8001

**Solution 2: Hardware reset:**

* Power cycle the TNC
* Press reset button if available
* Disconnect/reconnect USB

**Solution 3: Special escape sequence:**

Some TNCs support escape sequences:

.. code-block:: text

   +++
   (wait 1 second)
   CMD:

Performance Issues
------------------

High CPU Usage
~~~~~~~~~~~~~~

**Normal usage:** <5% CPU

**If high CPU usage:**

.. code-block:: bash

   # Check what's using CPU
   top -p $(pgrep rax25kb)
   
   # Disable verbose features
   rax25kb -D /dev/ttyUSB0 -L 5  # Lower log level
   # Don't use -d -a unless debugging

High Memory Usage
~~~~~~~~~~~~~~~~~

**Normal usage:** <10 MB

**If high memory:**

.. code-block:: bash

   # Check memory
   ps aux | grep rax25kb
   
   # Look for memory leaks
   valgrind --leak-check=full rax25kb -D /dev/ttyUSB0

Slow Performance
~~~~~~~~~~~~~~~~

**Possible causes:**

1. Serial buffer overflow
2. Network congestion
3. Disk I/O (logging)

**Solutions:**

.. code-block:: bash

   # Use hardware flow control
   rax25kb -D /dev/ttyUSB0 -H
   
   # Reduce logging
   rax25kb -D /dev/ttyUSB0 -L 5 --console-only
   
   # Check system load
   uptime
   vmstat 1

Application Integration Issues
-------------------------------

APRS Software Not Receiving Data
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Checklist:**

1. **rax25kb is running**
2. **TCP connection established**
3. **TNC in KISS mode**
4. **APRS software configured for KISS over TCP**

**Test connection:**

.. code-block:: bash

   telnet localhost 8001
   # Should connect successfully

**Verify APRS software settings:**

* Protocol: KISS or KISS over TCP
* Host: localhost (or IP)
* Port: 8001 (or configured port)

Direwolf Integration Issues
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Problem:** Direwolf can't connect to rax25kb

**Direwolf config:**

.. code-block:: ini

   KISSPORT 8001

**Start order:**

1. Start rax25kb first
2. Then start Direwolf

**Test:**

.. code-block:: bash

   # Terminal 1
   rax25kb -D /dev/ttyUSB0 -p 8001 -k
   
   # Terminal 2
   direwolf -c direwolf.conf -t 0

Xastir Connection Problems
~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Configuration:**

* Interface → Add → Network KISS
* Host: localhost
* Port: 8001
* Device name: rax25kb

**Common issues:**

* Wrong port number
* rax25kb not running
* Firewall blocking localhost

Diagnostic Tools
----------------

Enable Full Debugging
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 \
           -k -a -d \
           -L 9 \
           -l /tmp/rax25kb-debug.log \
           --pcap /tmp/packets.pcap

This enables:

* KISS frame parsing (``-k``)
* AX.25 info display (``-a``)
* Hex dump (``-d``)
* Maximum logging (``-L 9``)
* Log file
* Packet capture

Test Serial Port
~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Linux - test with screen
   screen /dev/ttyUSB0 9600
   
   # Linux - test with minicom
   minicom -D /dev/ttyUSB0 -b 9600
   
   # Test with rax25kb
   rax25kb -D /dev/ttyUSB0 -R -p 8001

Test TCP Connection
~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Test with telnet
   telnet localhost 8001
   
   # Test with netcat
   nc localhost 8001
   
   # Test with curl
   curl telnet://localhost:8001

Monitor Packets in Real-Time
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # With parsing
   rax25kb -D /dev/ttyUSB0 -k -a
   
   # With hex dump
   rax25kb -D /dev/ttyUSB0 -d
   
   # Save to log
   rax25kb -D /dev/ttyUSB0 -k -L 6 -l /tmp/packets.log
   tail -f /tmp/packets.log

Capture for Later Analysis
~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Capture packets
   rax25kb -D /dev/ttyUSB0 --pcap /tmp/packets.pcap
   
   # Analyze with Wireshark
   wireshark /tmp/packets.pcap
   
   # Or use tcpdump
   tcpdump -r /tmp/packets.pcap -X

Common Error Messages
---------------------

"Configuration file not found"
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Create default config
   cp examples/rax25kb.cfg .
   
   # Or specify path
   rax25kb -c /path/to/config.cfg

"Failed to create log file"
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Cause:** Insufficient permissions or directory doesn't exist

**Solution:**

.. code-block:: bash

   # Create directory
   sudo mkdir -p /var/log/rax25kb
   sudo chown $USER /var/log/rax25kb
   
   # Or use user-writable location
   rax25kb -l ~/rax25kb.log

"Invalid baud rate"
~~~~~~~~~~~~~~~~~~~

**Solution:** Use a standard baud rate:

* 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200

Getting Help
------------

Before Asking for Help
~~~~~~~~~~~~~~~~~~~~~~

1. Check this troubleshooting guide
2. Review the :doc:`faq`
3. Enable debug logging and capture output
4. Test with minimal configuration
5. Verify hardware connections

Information to Provide
~~~~~~~~~~~~~~~~~~~~~~

When reporting issues, include:

1. **Operating system and version**
2. **rax25kb version**
3. **Complete command line**
4. **Configuration file (if used)**
5. **Complete error messages**
6. **Debug log output**
7. **TNC make/model**
8. **What you've already tried**

Example bug report:

.. code-block:: text

   OS: Ubuntu 22.04 LTS
   rax25kb: 1.0.0
   Command: rax25kb -D /dev/ttyUSB0 -b 9600 -k
   TNC: Mobilinkd TNC3
   
   Error: Packets appear corrupted
   
   Debug output:
   [paste debug log]
   
   Tried:
   - Different baud rates
   - PhilFlag enabled
   - Verified TNC in KISS mode

Where to Get Help
~~~~~~~~~~~~~~~~~

* GitHub Issues: https://github.com/ke4ahr/rax25kb/issues
* Documentation: https://www.outpostpm.org/support.html
* Amateur radio forums
* Local amateur radio club

See Also
--------

* :doc:`faq` - Frequently asked questions
* :doc:`usage` - Usage guide
* :doc:`examples` - Working examples
* :doc:`configuration` - Configuration options
