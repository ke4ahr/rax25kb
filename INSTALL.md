# Installation Guide for rax25kb v1.7.3

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building from Source](#building-from-source)
- [Installation](#installation)
- [Platform-Specific Notes](#platform-specific-notes)
- [Configuration](#configuration)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### All Platforms

- **Rust Toolchain**: Version 1.70.0 or later
  - Install from: https://rustup.rs/

### Linux

- GCC or Clang compiler
- libudev development files
- pkg-config

**Debian/Ubuntu:**
```bash
sudo apt-get update
sudo apt-get install build-essential libudev-dev pkg-config
```

**Fedora/RHEL/CentOS:**
```bash
sudo dnf install gcc libudev-devel pkgconfig
```

**Arch Linux:**
```bash
sudo pacman -S base-devel systemd
```

### Windows

- Microsoft Visual C++ Build Tools
  - Install from: https://visualstudio.microsoft.com/downloads/
  - Select "Desktop development with C++"

### macOS

- Xcode Command Line Tools
```bash
xcode-select --install
```

## Building from Source

### 1. Clone the Repository

```bash
git clone https://github.com/ke4ahr/rax25kb.git
cd rax25kb
```

### 2. Build the Project

**Debug Build:**
```bash
cargo build
```

**Release Build (Recommended):**
```bash
cargo build --release
```

The compiled binary will be located at:
- Debug: `target/debug/rax25kb`
- Release: `target/release/rax25kb`

### 3. Run Tests (Optional)

```bash
cargo test
```

## Installation

### Linux

#### System-wide Installation

```bash
# Build release binary
cargo build --release

# Install binary
sudo install -m 755 target/release/rax25kb /usr/local/bin/

# Install man pages
sudo install -d /usr/local/share/man/man1
sudo install -d /usr/local/share/man/man5
sudo install -m 644 man/rax25kb.1 /usr/local/share/man/man1/
sudo install -m 644 man/rax25kb.cfg.5 /usr/local/share/man/man5/

# Update man database
sudo mandb
```

#### User Installation

```bash
# Build release binary
cargo build --release

# Install to user's local bin
mkdir -p ~/.local/bin
cp target/release/rax25kb ~/.local/bin/

# Add to PATH if not already (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.local/bin:$PATH"
```

#### Using systemd (Optional)

Create a systemd service file `/etc/systemd/system/rax25kb.service`:

```ini
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
```

Enable and start the service:
```bash
sudo systemctl daemon-reload
sudo systemctl enable rax25kb
sudo systemctl start rax25kb
```

### Windows

#### Manual Installation

1. Build the release binary:
```cmd
cargo build --release
```

2. Copy the executable to a directory in your PATH:
```cmd
copy target\release\rax25kb.exe C:\Windows\System32\
```

Or create a dedicated directory:
```cmd
mkdir C:\Program Files\rax25kb
copy target\release\rax25kb.exe "C:\Program Files\rax25kb\"
```

3. Add to PATH (optional):
   - Open System Properties → Environment Variables
   - Add `C:\Program Files\rax25kb` to PATH

#### Running as a Windows Service

Use NSSM (Non-Sucking Service Manager):

1. Download NSSM from https://nssm.cc/download
2. Install the service:
```cmd
nssm install rax25kb "C:\Program Files\rax25kb\rax25kb.exe"
nssm set rax25kb AppDirectory "C:\Program Files\rax25kb"
nssm set rax25kb AppParameters "-c C:\Program Files\rax25kb\rax25kb.cfg"
nssm set rax25kb DisplayName "rax25kb AX.25 KISS Bridge"
nssm set rax25kb Description "Multi-port KISS/XKISS bridge for AX.25 TNCs"
nssm set rax25kb Start SERVICE_AUTO_START
```

3. Start the service:
```cmd
nssm start rax25kb
```

### macOS

```bash
# Build release binary
cargo build --release

# Install binary
sudo cp target/release/rax25kb /usr/local/bin/

# Install man pages
sudo mkdir -p /usr/local/share/man/man1
sudo mkdir -p /usr/local/share/man/man5
sudo cp man/rax25kb.1 /usr/local/share/man/man1/
sudo cp man/rax25kb.cfg.5 /usr/local/share/man/man5/
```

#### Using launchd (Optional)

Create `/Library/LaunchDaemons/org.ke4ahr.rax25kb.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" 
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>org.ke4ahr.rax25kb</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/rax25kb</string>
        <string>-c</string>
        <string>/usr/local/etc/rax25kb.cfg</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Load the service:
```bash
sudo launchctl load /Library/LaunchDaemons/org.ke4ahr.rax25kb.plist
```

## Platform-Specific Notes

### Linux Serial Port Permissions

Add your user to the `dialout` group:
```bash
sudo usermod -a -G dialout $USER
```

Log out and back in for changes to take effect.

Alternatively, use udev rules. Create `/etc/udev/rules.d/99-serial.rules`:
```
KERNEL=="ttyUSB[0-9]*", MODE="0666"
KERNEL=="ttyACM[0-9]*", MODE="0666"
```

Reload udev rules:
```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### Windows COM Port Access

- Run as Administrator if you encounter permission issues
- Verify COM port is not in use by another application
- Check Device Manager for proper driver installation

### macOS Serial Port Access

Serial devices typically appear as `/dev/cu.usbserial-*` or `/dev/cu.SLAB_USBtoUART`.

Grant Full Disk Access if needed:
- System Preferences → Security & Privacy → Privacy → Full Disk Access
- Add Terminal or your application

## Configuration

### Create Configuration Directory

**Linux:**
```bash
sudo mkdir -p /etc/rax25kb
sudo cp doc/examples/rax25kb.cfg /etc/rax25kb/
```

**Windows:**
```cmd
mkdir "C:\Program Files\rax25kb"
copy doc\examples\rax25kb.cfg "C:\Program Files\rax25kb\"
```

**macOS:**
```bash
sudo mkdir -p /usr/local/etc
sudo cp doc/examples/rax25kb.cfg /usr/local/etc/
```

### Edit Configuration

Edit the configuration file for your setup. See `man rax25kb.cfg` or the documentation for details.

**Example minimal configuration:**
```ini
# Single serial to TCP bridge
cross_connect0000.serial_port=/dev/ttyUSB0
cross_connect0000.baud_rate=9600
cross_connect0000.tcp_address=0.0.0.0
cross_connect0000.tcp_port=8001
```

## Verification

### Test the Installation

```bash
# Show version and help
rax25kb --help

# Test with config file (foreground)
rax25kb -c /path/to/rax25kb.cfg

# Check man pages
man rax25kb
man rax25kb.cfg
```

### Verify Serial Port Access

**Linux:**
```bash
ls -l /dev/ttyUSB* /dev/ttyACM*
```

**Windows:**
```cmd
mode
```

**macOS:**
```bash
ls -l /dev/cu.*
```

## Troubleshooting

### Common Issues

#### "Permission denied" on serial port

**Linux:**
- Add user to dialout group
- Check udev rules
- Verify port permissions with `ls -l /dev/ttyUSB0`

**Windows:**
- Run as Administrator
- Check if another application is using the port
- Verify driver installation

#### "Address already in use"

- Another instance is running
- Another application is using the TCP port
- Change `tcp_port` in configuration

#### "No such file or directory" for serial port

- Verify device is connected: `lsusb` (Linux) or Device Manager (Windows)
- Check correct port name
- Verify driver installation

#### Build Errors

**libudev not found (Linux):**
```bash
sudo apt-get install libudev-dev
```

**MSVC not found (Windows):**
- Install Visual Studio Build Tools
- Run from "Developer Command Prompt for VS"

### Getting Help

- Documentation: https://github.com/ke4ahr/rax25kb/
- Man pages: `man rax25kb`, `man rax25kb.cfg`
- Issues: https://github.com/ke4ahr/rax25kb/issues

### Logs and Debugging

Enable verbose logging:
```bash
rax25kb -c config.cfg -L 9 -l /var/log/rax25kb.log
```

Check system logs:

**Linux:**
```bash
journalctl -u rax25kb -f
```

**Windows:**
```cmd
# Check service logs with Event Viewer
eventvwr.msc
```

## Uninstallation

### Linux

```bash
# Remove binary
sudo rm /usr/local/bin/rax25kb

# Remove man pages
sudo rm /usr/local/share/man/man1/rax25kb.1
sudo rm /usr/local/share/man/man5/rax25kb.cfg.5
sudo mandb

# Remove configuration (optional)
sudo rm -rf /etc/rax25kb

# Remove systemd service (if installed)
sudo systemctl stop rax25kb
sudo systemctl disable rax25kb
sudo rm /etc/systemd/system/rax25kb.service
sudo systemctl daemon-reload
```

### Windows

```cmd
# Stop and remove service (if installed)
nssm stop rax25kb
nssm remove rax25kb confirm

# Remove files
rmdir /s "C:\Program Files\rax25kb"
```

### macOS

```bash
# Stop launchd service (if installed)
sudo launchctl unload /Library/LaunchDaemons/org.ke4ahr.rax25kb.plist
sudo rm /Library/LaunchDaemons/org.ke4ahr.rax25kb.plist

# Remove binary and man pages
sudo rm /usr/local/bin/rax25kb
sudo rm /usr/local/share/man/man1/rax25kb.1
sudo rm /usr/local/share/man/man5/rax25kb.cfg.5

# Remove configuration (optional)
sudo rm -rf /usr/local/etc/rax25kb.cfg
```

## Next Steps

After installation:

1. Read the documentation: `man rax25kb`
2. Configure your bridges: `man rax25kb.cfg`
3. See example configurations in `doc/examples/`
4. Review the ARCHITECTURE.md for system design
5. Join the community at https://github.com/ke4ahr/rax25kb/

