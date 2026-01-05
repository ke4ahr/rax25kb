Windows Usage Guide
===================

This guide provides Windows-specific instructions for installing, configuring, and using rax25kb.

Prerequisites
-------------

Windows Version
~~~~~~~~~~~~~~~

* Windows 10 or later (64-bit recommended)
* Windows Server 2016 or later

Development Tools
~~~~~~~~~~~~~~~~~

For building from source:

* **Rust Toolchain**: Install from https://rustup.rs/
  
  * Select "1) Proceed with installation (default)"
  * This installs the MSVC toolchain automatically

* **Visual Studio Build Tools** (if needed):
  
  * Download from https://visualstudio.microsoft.com/downloads/
  * Install "Desktop development with C++"

Serial Port Drivers
~~~~~~~~~~~~~~~~~~~

Most USB-to-serial adapters require drivers:

* **FTDI Chips**: Download from https://ftdichip.com/drivers/
* **Prolific PL2303**: Download from http://www.prolific.com.tw/
* **Silicon Labs CP210x**: Download from https://www.silabs.com/
* **Built-in Windows Driver**: Many devices work with built-in driver

Installation
------------

Option 1: Pre-built Binary
~~~~~~~~~~~~~~~~~~~~~~~~~~~

1. Download the latest release from GitHub
2. Extract to a folder (e.g., ``C:\Program Files\rax25kb``)
3. Add to PATH (optional):

   * Open System Properties → Environment Variables
   * Edit PATH variable
   * Add ``C:\Program Files\rax25kb``

Option 2: Build from Source
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: cmd

    # Open Command Prompt or PowerShell
    git clone https://github.com/ke4ahr/rax25kb.git
    cd rax25kb
    cargo build --release
    
    # Binary is at: target\release\rax25kb.exe

COM Port Configuration
----------------------

Identifying COM Ports
~~~~~~~~~~~~~~~~~~~~~

Use Device Manager:

1. Press ``Win + X`` → Device Manager
2. Expand "Ports (COM & LPT)"
3. Note your COM port number (e.g., COM3, COM5)

Or use command line:

.. code-block:: cmd

    mode

Testing COM Port Access
~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: cmd

    # List available ports
    mode
    
    # Configure port (optional)
    mode COM3 BAUD=9600 PARITY=N DATA=8 STOP=1

Configuration File
------------------

Location
~~~~~~~~

Recommended locations:

* ``C:\Program Files\rax25kb\rax25kb.cfg``
* ``C:\ProgramData\rax25kb\rax25kb.cfg``
* Current directory: ``rax25kb.cfg``

Example Configuration
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: ini

    # Simple configuration for Windows
    cross_connect0000.serial_port=COM3
    cross_connect0000.baud_rate=9600
    cross_connect0000.tcp_address=0.0.0.0
    cross_connect0000.tcp_port=8001
    cross_connect0000.phil_flag=yes
    
    log_level=5
    logfile=C:\\ProgramData\\rax25kb\\rax25kb.log
    pidfile=C:\\ProgramData\\rax25kb\\rax25kb.pid

.. note::
   Use double backslashes (``\\``) in paths or forward slashes (``/``)

Path Format
~~~~~~~~~~~

Windows paths can be written in two ways:

.. code-block:: ini

    # Double backslashes
    logfile=C:\\ProgramData\\rax25kb\\log.txt
    
    # Forward slashes (recommended)
    logfile=C:/ProgramData/rax25kb/log.txt

Running rax25kb
---------------

Command Prompt
~~~~~~~~~~~~~~

.. code-block:: cmd

    # Change to program directory
    cd "C:\Program Files\rax25kb"
    
    # Run with default config
    rax25kb.exe
    
    # Run with specific config
    rax25kb.exe -c "C:\ProgramData\rax25kb\rax25kb.cfg"
    
    # Run with options
    rax25kb.exe -D COM3 -b 9600 -p 8001 -n

PowerShell
~~~~~~~~~~

.. code-block:: powershell

    # Run with default config
    .\rax25kb.exe
    
    # Run with specific config
    .\rax25kb.exe -c "C:\ProgramData\rax25kb\rax25kb.cfg"
    
    # Run in background (PowerShell 7+)
    Start-Process -NoNewWindow rax25kb.exe -ArgumentList "-c config.cfg"

Running as Administrator
~~~~~~~~~~~~~~~~~~~~~~~~

Some operations may require administrator privileges:

.. code-block:: cmd

    # Right-click Command Prompt → Run as Administrator
    cd "C:\Program Files\rax25kb"
    rax25kb.exe -c rax25kb.cfg

Running as a Windows Service
-----------------------------

Using NSSM
~~~~~~~~~~

**NSSM** (Non-Sucking Service Manager) is recommended for running rax25kb as a service.

1. Download NSSM from https://nssm.cc/download
2. Extract nssm.exe to a folder
3. Install as service:

.. code-block:: cmd

    # Open Command Prompt as Administrator
    cd C:\nssm
    
    # Install service
    nssm install rax25kb "C:\Program Files\rax25kb\rax25kb.exe"
    
    # Configure service
    nssm set rax25kb AppDirectory "C:\Program Files\rax25kb"
    nssm set rax25kb AppParameters "-c C:\ProgramData\rax25kb\rax25kb.cfg"
    nssm set rax25kb DisplayName "rax25kb AX.25 KISS Bridge"
    nssm set rax25kb Description "Multi-port KISS/XKISS bridge for AX.25 TNCs"
    nssm set rax25kb Start SERVICE_AUTO_START
    nssm set rax25kb AppStdout "C:\ProgramData\rax25kb\stdout.log"
    nssm set rax25kb AppStderr "C:\ProgramData\rax25kb\stderr.log"
    
    # Start service
    nssm start rax25kb

Service Management
~~~~~~~~~~~~~~~~~~

.. code-block:: cmd

    # Start service
    nssm start rax25kb
    # or: net start rax25kb
    
    # Stop service
    nssm stop rax25kb
    # or: net stop rax25kb
    
    # Restart service
    nssm restart rax25kb
    
    # Remove service
    nssm remove rax25kb confirm

Using sc.exe
~~~~~~~~~~~~

Alternative method using built-in Windows service controller:

.. code-block:: cmd

    # Create service
    sc create rax25kb binPath= "C:\Program Files\rax25kb\rax25kb.exe -c C:\ProgramData\rax25kb\rax25kb.cfg" start= auto
    
    # Start service
    sc start rax25kb
    
    # Stop service
    sc stop rax25kb
    
    # Delete service
    sc delete rax25kb

Firewall Configuration
----------------------

Windows Firewall
~~~~~~~~~~~~~~~~

Allow rax25kb through Windows Firewall:

.. code-block:: cmd

    # Add inbound rule for TCP port 8001
    netsh advfirewall firewall add rule name="rax25kb" dir=in action=allow protocol=TCP localport=8001

Or use GUI:

1. Windows Security → Firewall & network protection
2. Advanced settings → Inbound Rules → New Rule
3. Port → TCP → Specific ports: 8001
4. Allow the connection
5. Apply to all profiles
6. Name: "rax25kb"

Common Windows Issues
---------------------

COM Port Access Denied
~~~~~~~~~~~~~~~~~~~~~~~

**Solution**:

* Close other applications using the port
* Run as Administrator
* Check Device Manager for conflicts
* Verify drivers are installed

COM Port Not Found
~~~~~~~~~~~~~~~~~~

**Solution**:

* Check Device Manager for port
* Reinstall USB-to-serial drivers
* Try different USB port
* Check cable connections

TCP Port Already in Use
~~~~~~~~~~~~~~~~~~~~~~~

**Solution**:

.. code-block:: cmd

    # Find what's using the port
    netstat -ano | findstr :8001
    
    # Kill the process (replace PID)
    taskkill /PID <PID> /F

Permission Errors
~~~~~~~~~~~~~~~~~

**Solution**:

* Run Command Prompt as Administrator
* Check folder permissions
* Use folders like ``C:\ProgramData`` for logs
* Avoid ``C:\Program Files`` for writable files

Service Won't Start
~~~~~~~~~~~~~~~~~~~

**Solution**:

* Check Event Viewer: Win + X → Event Viewer → Windows Logs → Application
* Verify paths in service configuration
* Check log files for errors
* Ensure config file exists and is valid

Example Configurations
----------------------

Single TNC
~~~~~~~~~~

.. code-block:: ini

    # Kantronics KPC-3 on COM3
    cross_connect0000.serial_port=COM3
    cross_connect0000.baud_rate=9600
    cross_connect0000.tcp_address=127.0.0.1
    cross_connect0000.tcp_port=8001
    cross_connect0000.phil_flag=yes
    
    log_level=5
    logfile=C:/ProgramData/rax25kb/rax25kb.log

Multiple TNCs
~~~~~~~~~~~~~

.. code-block:: ini

    # TNC 1 on COM3
    cross_connect0000.serial_port=COM3
    cross_connect0000.baud_rate=9600
    cross_connect0000.tcp_address=0.0.0.0
    cross_connect0000.tcp_port=8001
    
    # TNC 2 on COM5
    cross_connect0001.serial_port=COM5
    cross_connect0001.baud_rate=19200
    cross_connect0001.tcp_address=0.0.0.0
    cross_connect0001.tcp_port=8002
    
    log_level=6
    logfile=C:/ProgramData/rax25kb/rax25kb.log

With AGWPE Compatibility
~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: ini

    # Compatible with AGWPE applications
    cross_connect0000.serial_port=COM3
    cross_connect0000.baud_rate=9600
    cross_connect0000.tcp_address=127.0.0.1
    cross_connect0000.tcp_port=8000
    cross_connect0000.phil_flag=no
    cross_connect0000.parse_kiss=yes
    
    log_level=5

Testing on Windows
------------------

Using PuTTY
~~~~~~~~~~~

1. Download PuTTY from https://www.putty.org/
2. Open PuTTY
3. Connection type: Raw
4. Host: localhost, Port: 8001
5. Click "Open"

Using PowerShell
~~~~~~~~~~~~~~~~

.. code-block:: powershell

    # Test TCP connection
    Test-NetConnection -ComputerName localhost -Port 8001
    
    # Connect with PowerShell (requires PowerShell 6+)
    $client = New-Object System.Net.Sockets.TcpClient("localhost", 8001)
    $stream = $client.GetStream()

Monitoring
----------

Task Manager
~~~~~~~~~~~~

1. Press ``Ctrl + Shift + Esc``
2. Details tab → Find ``rax25kb.exe``
3. Monitor CPU and memory usage

Resource Monitor
~~~~~~~~~~~~~~~~

1. Press ``Win + R`` → ``resmon``
2. Network tab → TCP Connections
3. Look for rax25kb connections

Event Viewer
~~~~~~~~~~~~

1. Press ``Win + X`` → Event Viewer
2. Windows Logs → Application
3. Filter for "rax25kb" events

Uninstallation
--------------

Service Removal
~~~~~~~~~~~~~~~

.. code-block:: cmd

    # Stop and remove NSSM service
    nssm stop rax25kb
    nssm remove rax25kb confirm
    
    # Or use sc
    sc stop rax25kb
    sc delete rax25kb

File Removal
~~~~~~~~~~~~

.. code-block:: cmd

    # Remove program files
    rmdir /s "C:\Program Files\rax25kb"
    
    # Remove data files (optional)
    rmdir /s "C:\ProgramData\rax25kb"

Firewall Rule Removal
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: cmd

    netsh advfirewall firewall delete rule name="rax25kb"

Additional Resources
--------------------

* Windows Serial Port Programming: https://docs.microsoft.com/en-us/windows/win32/devio/communications-resources
* NSSM Documentation: https://nssm.cc/usage
* PowerShell Documentation: https://docs.microsoft.com/en-us/powershell/
