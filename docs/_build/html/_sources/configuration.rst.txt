Configuration
=============

Configuration File
------------------

The default configuration file is ``rax25kb.cfg`` in the current directory. You can specify a different file using the ``-c`` option.

File Format
~~~~~~~~~~~

The configuration file uses a simple ``key=value`` format:

.. code-block:: ini

   # Comments start with #
   serial_port=/dev/ttyUSB0
   baud_rate=9600
   tcp_port=8001
   
   # Values can be quoted
   tcp_address="0.0.0.0 ::"

Configuration Options
~~~~~~~~~~~~~~~~~~~~~

Serial Port Settings
^^^^^^^^^^^^^^^^^^^^

**serial_port** (required)
   Serial port device path.
   
   * Linux: ``/dev/ttyUSB0``, ``/dev/ttyACM0``
   * Windows: ``COM3``, ``COM4``
   * macOS: ``/dev/cu.usbserial-*``

   Example:
   
   .. code-block:: ini
   
      serial_port=/dev/ttyUSB0

**baud_rate** (default: 9600)
   Serial port baud rate. Common values: 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200
   
   Example:
   
   .. code-block:: ini
   
      baud_rate=9600

**stop_bits** (default: 1)
   Number of stop bits. Valid values: ``1``, ``2``, ``one``, ``two``
   
   KISS TNC standard: 1
   
   Example:
   
   .. code-block:: ini
   
      stop_bits=1

**parity** (default: none)
   Parity checking. Valid values: ``none``, ``n``, ``no``, ``even``, ``e``, ``odd``, ``o``
   
   KISS TNC standard: none
   
   Example:
   
   .. code-block:: ini
   
      parity=none

**flow_control** (default: none)
   Serial port flow control. Valid values:
   
   * ``none``, ``off``, ``no`` - No flow control
   * ``software``, ``xon``, ``xonxoff``, ``xon-xoff`` - Software flow control (XON/XOFF)
   * ``hardware``, ``rtscts``, ``rts-cts``, ``rts/cts`` - Hardware flow control (RTS/CTS)
   * ``dtrdsr``, ``dtr-dsr``, ``dtr/dsr`` - DTR/DSR flow control
   
   Example:
   
   .. code-block:: ini
   
      flow_control=none

Network Settings
^^^^^^^^^^^^^^^^

**tcp_address** (default: 0.0.0.0)
   TCP listener address(es). Multiple addresses can be space or comma separated.
   
   * ``0.0.0.0`` - All IPv4 interfaces
   * ``127.0.0.1`` - IPv4 localhost only
   * ``::`` - All IPv6 interfaces
   * ``::1`` - IPv6 localhost only
   * ``0.0.0.0 ::`` - Both IPv4 and IPv6
   
   Example:
   
   .. code-block:: ini
   
      tcp_address="0.0.0.0 ::"

**tcp_port** (default: 8001)
   TCP listening port. Valid range: 1-65535
   
   Example:
   
   .. code-block:: ini
   
      tcp_port=8001

Feature Flags
^^^^^^^^^^^^^

**phil_flag** (default: 0)
   Enable PhilFlag correction for TASCO modem chipset bugs.
   Valid values: ``0``, ``1``, ``true``, ``false``, ``yes``, ``no``
   
   See :doc:`philflag` for details.
   
   Example:
   
   .. code-block:: ini
   
      phil_flag=1

**dump** (default: 0)
   Enable hexadecimal frame dumping.
   Valid values: ``0``, ``1``, ``true``, ``false``, ``yes``, ``no``
   
   Example:
   
   .. code-block:: ini
   
      dump=0

**parse_kiss** (default: 0)
   Enable KISS frame parsing and display.
   Valid values: ``0``, ``1``, ``true``, ``false``, ``yes``, ``no``
   
   Example:
   
   .. code-block:: ini
   
      parse_kiss=1

**dump_ax25** (default: 0)
   Enable AX.25 information field dumping.
   Valid values: ``0``, ``1``, ``true``, ``false``, ``yes``, ``no``
   
   Example:
   
   .. code-block:: ini
   
      dump_ax25=0

**raw_copy** (default: 0)
   Enable raw copy mode (transparent pass-through).
   Valid values: ``0``, ``1``, ``true``, ``false``, ``yes``, ``no``
   
   When enabled, all KISS processing is disabled.
   
   Example:
   
   .. code-block:: ini
   
      raw_copy=0

Logging Settings
^^^^^^^^^^^^^^^^

**log_level** (default: 5)
   Logging verbosity level (0-9):
   
   * 0 - EMERG (system unusable)
   * 1 - ALERT (immediate action required)
   * 2 - CRIT (critical conditions)
   * 3 - ERROR (error conditions)
   * 4 - WARN (warning conditions)
   * 5 - NOTICE (normal but significant)
   * 6 - INFO (informational messages)
   * 7 - DEBUG (debug-level messages)
   * 8 - TRACE (detailed trace messages)
   * 9 - VERBOSE (maximum verbosity)
   
   Example:
   
   .. code-block:: ini
   
      log_level=5

**logfile** (default: none)
   Path to log file. If not specified, only console logging is used.
   
   Example:
   
   .. code-block:: ini
   
      logfile=/var/log/rax25kb.log

Output Settings
^^^^^^^^^^^^^^^

**pidfile** (default: none)
   Path to PID file. Process ID is written on startup.
   
   Example:
   
   .. code-block:: ini
   
      pidfile=/var/run/rax25kb.pid

**pcap_file** (default: none)
   Path to PCAP capture file. Only AX.25 data frames are captured.
   
   Example:
   
   .. code-block:: ini
   
      pcap_file=/var/log/rax25kb.pcap

Example Configuration Files
----------------------------

Basic KISS TNC Setup
~~~~~~~~~~~~~~~~~~~~

.. code-block:: ini

   # Basic KISS TNC configuration
   # Standard settings: 9600 baud, 8N1, no flow control
   
   serial_port=/dev/ttyUSB0
   baud_rate=9600
   stop_bits=1
   parity=none
   flow_control=none
   
   tcp_address=0.0.0.0
   tcp_port=8001
   
   log_level=5

Advanced Configuration with PhilFlag
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: ini

   # Advanced configuration for TASCO modem with PhilFlag
   
   serial_port=/dev/ttyUSB0
   baud_rate=9600
   stop_bits=1
   parity=none
   flow_control=none
   
   # Listen on both IPv4 and IPv6
   tcp_address="0.0.0.0 ::"
   tcp_port=8001
   
   # Enable PhilFlag correction
   phil_flag=1
   
   # Enable KISS parsing and AX.25 decoding
   parse_kiss=1
   dump_ax25=1
   
   # Logging configuration
   log_level=6
   logfile=/var/log/rax25kb.log
   
   # Capture packets
   pcap_file=/var/log/rax25kb.pcap
   
   # PID file for service management
   pidfile=/var/run/rax25kb.pid

High Speed TNC with Hardware Flow Control
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: ini

   # High-speed TNC configuration
   
   serial_port=/dev/ttyUSB0
   baud_rate=115200
   stop_bits=1
   parity=none
   flow_control=hardware
   
   tcp_address=127.0.0.1
   tcp_port=8001
   
   parse_kiss=1
   log_level=5

Windows Configuration
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: ini

   # Windows configuration
   
   serial_port=COM3
   baud_rate=9600
   stop_bits=1
   parity=none
   flow_control=none
   
   tcp_address=127.0.0.1
   tcp_port=8001
   
   logfile=C:\ProgramData\rax25kb\rax25kb.log
   pidfile=C:\ProgramData\rax25kb\rax25kb.pid
   
   log_level=5

Command-Line Overrides
----------------------

Command-line options override configuration file settings.

Serial Port Options
~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -D /dev/ttyUSB0 -b 19200 -s 1 -Q none -N

Network Options
~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -I "0.0.0.0 ::" -p 8001 -4  # IPv4 only
   rax25kb -I "0.0.0.0 ::" -p 8001 -6  # IPv6 only

Feature Options
~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -d          # Enable frame dumping
   rax25kb -k          # Enable KISS parsing
   rax25kb -a          # Enable AX.25 info dumping
   rax25kb -n          # Enable PhilFlag
   rax25kb -R          # Enable raw copy mode

Logging Options
~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -l /var/log/rax25kb.log -L 7
   rax25kb --console-only
   rax25kb --no-console -l /var/log/rax25kb.log

Complete Override Example
~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   rax25kb -c myconfig.cfg \
           -D /dev/ttyUSB0 \
           -b 9600 \
           -I "0.0.0.0" \
           -p 8001 \
           -n \
           -k \
           -l /var/log/rax25kb.log \
           -L 6 \
           --pcap /var/log/packets.pcap

Configuration Validation
------------------------

Test Configuration
~~~~~~~~~~~~~~~~~~

Test your configuration without starting the server:

.. code-block:: bash

   rax25kb -q  # Quiet mode - will exit if config is invalid

View Active Configuration
~~~~~~~~~~~~~~~~~~~~~~~~~~

Start without ``-q`` flag to see active configuration:

.. code-block:: bash

   rax25kb

Output will show:

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

Common Configuration Patterns
------------------------------

Localhost Only (Development)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: ini

   tcp_address=127.0.0.1
   tcp_port=8001

Public Access (Production)
~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: ini

   tcp_address=0.0.0.0
   tcp_port=8001

Dual Stack (IPv4 + IPv6)
~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: ini

   tcp_address="0.0.0.0 ::"
   tcp_port=8001

Debug Mode
~~~~~~~~~~

.. code-block:: ini

   parse_kiss=1
   dump_ax25=1
   log_level=7
   logfile=/var/log/rax25kb-debug.log

Production Mode
~~~~~~~~~~~~~~~

.. code-block:: ini

   log_level=5
   logfile=/var/log/rax25kb.log
   pidfile=/var/run/rax25kb.pid

Next Steps
----------

* :doc:`usage` - Learn how to use rax25kb
* :doc:`examples` - See practical examples
* :doc:`troubleshooting` - Solve common issues

