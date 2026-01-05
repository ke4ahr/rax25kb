Usage Guide
===========

This guide provides practical examples for using rax25kb in various scenarios.

Basic Usage
-----------

Command Line
~~~~~~~~~~~~

Display help:

.. code-block:: bash

    rax25kb --help

Run with configuration file:

.. code-block:: bash

    rax25kb -c /etc/rax25kb/rax25kb.cfg

Run with command-line options:

.. code-block:: bash

    rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001

Quick Start Examples
--------------------

Simple KISS Bridge
~~~~~~~~~~~~~~~~~~

Bridge a single KISS TNC to TCP port 8001:

.. code-block:: bash

    rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001

With PhilFlag correction:

.. code-block:: bash

    rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001 -n

With KISS parsing and logging:

.. code-block:: bash

    rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001 -k -l /var/log/rax25kb.log

Configuration File Usage
------------------------

Create a Configuration File
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Create ``/etc/rax25kb/rax25kb.cfg``:

.. code-block:: ini

    # Single TNC configuration
    cross_connect0000.serial_port=/dev/ttyUSB0
    cross_connect0000.baud_rate=9600
    cross_connect0000.tcp_address=0.0.0.0
    cross_connect0000.tcp_port=8001
    cross_connect0000.phil_flag=yes
    
    log_level=5
    logfile=/var/log/rax25kb.log

Run with the configuration:

.. code-block:: bash

    rax25kb -c /etc/rax25kb/rax25kb.cfg

Common Scenarios
----------------

Scenario 1: Single TNC Gateway
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Goal**: Provide network access to one KISS TNC

**Configuration**:

.. code-block:: ini

    cross_connect0000.serial_port=/dev/ttyUSB0
    cross_connect0000.baud_rate=9600
    cross_connect0000.tcp_address=0.0.0.0
    cross_connect0000.tcp_port=8001
    cross_connect0000.parse_kiss=yes
    
    log_level=6
    logfile=/var/log/rax25kb.log

**Command**:

.. code-block:: bash

    rax25kb -c rax25kb.cfg

Scenario 2: Multiple TNCs
~~~~~~~~~~~~~~~~~~~~~~~~~~

**Goal**: Bridge three TNCs to different TCP ports

**Configuration**:

.. code-block:: ini

    # VHF TNC
    cross_connect0000.serial_port=/dev/ttyUSB0
    cross_connect0000.baud_rate=9600
    cross_connect0000.tcp_address=0.0.0.0
    cross_connect0000.tcp_port=8001
    
    # UHF TNC
    cross_connect0001.serial_port=/dev/ttyUSB1
    cross_connect0001.baud_rate=9600
    cross_connect0001.tcp_address=0.0.0.0
    cross_connect0001.tcp_port=8002
    
    # HF TNC
    cross_connect0002.serial_port=/dev/ttyUSB2
    cross_connect0002.baud_rate=19200
    cross_connect0002.tcp_address=0.0.0.0
    cross_connect0002.tcp_port=8003
    
    log_level=5
    logfile=/var/log/rax25kb.log

Scenario 3: KISS to XKISS Translation
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Goal**: Connect standard KISS port 0 to XKISS port 5

**Configuration**:

.. code-block:: ini

    # KISS TNC on port 0
    cross_connect0000.serial_port=/dev/ttyUSB0
    cross_connect0000.baud_rate=9600
    cross_connect0000.kiss_port=0
    cross_connect0000.serial_to_serial=/dev/ttyUSB1
    
    # XKISS TNC on port 5
    cross_connect0001.serial_port=/dev/ttyUSB1
    cross_connect0001.baud_rate=9600
    cross_connect0001.xkiss_mode=yes
    cross_connect0001.xkiss_port=5
    
    log_level=7

Scenario 4: Packet Capture
~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Goal**: Capture all AX.25 frames to PCAP file

**Configuration**:

.. code-block:: ini

    cross_connect0000.serial_port=/dev/ttyUSB0
    cross_connect0000.baud_rate=9600
    cross_connect0000.tcp_address=127.0.0.1
    cross_connect0000.tcp_port=8001
    cross_connect0000.parse_kiss=yes
    
    log_level=6
    pcap_file=/var/log/rax25kb.pcap

**Analysis**:

.. code-block:: bash

    # View with tcpdump
    tcpdump -r /var/log/rax25kb.pcap -X
    
    # View with Wireshark
    wireshark /var/log/rax25kb.pcap

Scenario 5: Development/Testing
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Goal**: Debug KISS communication with verbose logging

**Configuration**:

.. code-block:: ini

    cross_connect0000.serial_port=/dev/ttyUSB0
    cross_connect0000.baud_rate=9600
    cross_connect0000.tcp_address=127.0.0.1
    cross_connect0000.tcp_port=8001
    cross_connect0000.dump=yes
    cross_connect0000.parse_kiss=yes
    cross_connect0000.dump_ax25=yes
    
    log_level=9
    logfile=/tmp/rax25kb-debug.log
    log_to_console=yes

Running as a Daemon
-------------------

Linux systemd
~~~~~~~~~~~~~

Create ``/etc/systemd/system/rax25kb.service``:

.. code-block:: ini

    [Unit]
    Description=rax25kb AX.25 KISS Bridge
    After=network.target
    
    [Service]
    Type=simple
    User=radio
    Group=dialout
    WorkingDirectory=/etc/rax25kb
    ExecStart=/usr/local/bin/rax25kb -c /etc/rax25kb/rax25kb.cfg
    Restart=on-failure
    RestartSec=5s
    
    [Install]
    WantedBy=multi-user.target

Enable and start:

.. code-block:: bash

    sudo systemctl daemon-reload
    sudo systemctl enable rax25kb
    sudo systemctl start rax25kb
    sudo systemctl status rax25kb

View logs:

.. code-block:: bash

    journalctl -u rax25kb -f

Background Process
~~~~~~~~~~~~~~~~~~

Simple background execution:

.. code-block:: bash

    # Run in background
    rax25kb -c /etc/rax25kb/rax25kb.cfg &
    
    # With nohup
    nohup rax25kb -c /etc/rax25kb/rax25kb.cfg > /dev/null 2>&1 &
    
    # Save PID
    echo $! > /var/run/rax25kb.pid

Stop background process:

.. code-block:: bash

    # Using saved PID
    kill $(cat /var/run/rax25kb.pid)
    
    # Or find and kill
    pkill rax25kb

Using screen
~~~~~~~~~~~~

.. code-block:: bash

    # Start in screen session
    screen -S rax25kb
    rax25kb -c /etc/rax25kb/rax25kb.cfg
    
    # Detach: Ctrl+A, D
    
    # Reattach
    screen -r rax25kb

Using tmux
~~~~~~~~~~

.. code-block:: bash

    # Start in tmux session
    tmux new -s rax25kb
    rax25kb -c /etc/rax25kb/rax25kb.cfg
    
    # Detach: Ctrl+B, D
    
    # Reattach
    tmux attach -t rax25kb

Monitoring
----------

Check Running Status
~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

    # Check if running
    ps aux | grep rax25kb
    
    # Check TCP port
    netstat -tulpn | grep 8001
    
    # Or with ss
    ss -tulpn | grep 8001

View Logs
~~~~~~~~~

.. code-block:: bash

    # Tail log file
    tail -f /var/log/rax25kb.log
    
    # View systemd logs
    journalctl -u rax25kb -f
    
    # Last 100 lines
    tail -n 100 /var/log/rax25kb.log

Test Connection
~~~~~~~~~~~~~~~

.. code-block:: bash

    # Test with telnet
    telnet localhost 8001
    
    # Test with netcat
    nc localhost 8001
    
    # Test with socat
    socat - TCP:localhost:8001

Troubleshooting Commands
------------------------

Check Serial Port
~~~~~~~~~~~~~~~~~

.. code-block:: bash

    # List serial ports
    ls -l /dev/ttyUSB* /dev/ttyACM*
    
    # Check permissions
    ls -l /dev/ttyUSB0
    
    # Test with minicom
    minicom -D /dev/ttyUSB0 -b 9600

Check TCP Port
~~~~~~~~~~~~~~

.. code-block:: bash

    # Check if port is listening
    netstat -tulpn | grep 8001
    
    # Check if port is in use
    lsof -i :8001
    
    # Test connection
    telnet localhost 8001

Verify Configuration
~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

    # Syntax check (dry run)
    rax25kb -c /etc/rax25kb/rax25kb.cfg -q
    
    # Verbose startup
    rax25kb -c /etc/rax25kb/rax25kb.cfg -L 9

Performance Monitoring
----------------------

System Resources
~~~~~~~~~~~~~~~~

.. code-block:: bash

    # CPU and memory usage
    top -p $(pgrep rax25kb)
    
    # Detailed stats
    htop -p $(pgrep rax25kb)

Network Statistics
~~~~~~~~~~~~~~~~~~

.. code-block:: bash

    # Connection count
    netstat -an | grep 8001 | wc -l
    
    # Traffic statistics
    iftop -i any -f "port 8001"

Serial Port Statistics
~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

    # Port statistics (if supported)
    stty -F /dev/ttyUSB0 -a
    
    # Monitor with setserial
    setserial -a /dev/ttyUSB0

Advanced Usage
--------------

Custom Log Rotation
~~~~~~~~~~~~~~~~~~~

Create ``/etc/logrotate.d/rax25kb``:

.. code-block:: text

    /var/log/rax25kb.log {
        daily
        rotate 7
        compress
        delaycompress
        missingok
        notifempty
        create 0640 radio dialout
        postrotate
            systemctl reload rax25kb > /dev/null 2>&1 || true
        endscript
    }

Integration with APRS
~~~~~~~~~~~~~~~~~~~~~

Using with Direwolf:

.. code-block:: bash

    # Start rax25kb
    rax25kb -D /dev/ttyUSB0 -p 8001
    
    # Configure Direwolf to connect to localhost:8001
    # In direwolf.conf:
    # KISSTNC localhost:8001

Integration with Other Software
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* **AGWPE Compatible**: Set TCP port to match AGWPE settings
* **soundmodem**: Connect via TCP to rax25kb port
* **LinBPQ**: Configure as KISS port in bpq32.cfg
* **UZ7HO Soundmodem**: Use TCP KISS option

Best Practices
--------------

1. **Use Configuration Files**: More maintainable than command-line options
2. **Enable Logging**: Essential for troubleshooting
3. **Set Appropriate Log Levels**: Level 5-6 for production, 7-9 for debugging
4. **Use systemd**: Automatic restart on failure
5. **Monitor Logs**: Check regularly for errors
6. **Test Changes**: Verify configuration before deploying
7. **Document Setup**: Keep notes on your configuration
8. **Regular Backups**: Backup configuration files
9. **Update Regularly**: Keep rax25kb up to date
10. **Use PID Files**: Makes management easier

See Also
--------

* :doc:`configuration` - Configuration file format
* :doc:`troubleshooting` - Common issues and solutions
* :doc:`windows` - Windows-specific usage
