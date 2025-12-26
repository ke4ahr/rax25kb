# Installation Guide

This document provides detailed installation instructions for rax25kb on various platforms.

## Table of Contents

- [System Requirements](#system-requirements)
- [Quick Install](#quick-install)
- [Building from Source](#building-from-source)
- [Platform-Specific Instructions](#platform-specific-instructions)
- [Post-Installation](#post-installation)
- [Troubleshooting](#troubleshooting)

## System Requirements

### Minimum Requirements

- **Operating System**: Linux (kernel 2.6+), Windows 7+, macOS 10.12+, or BSD
- **RAM**: 64 MB
- **Disk Space**: 50 MB (including source and build files)
- **Serial Port**: Physical serial port or USB-to-serial adapter

### Software Dependencies

#### For Binary Installation
- None (statically linked binary)

#### For Building from Source
- Rust 2021 edition or later (1.56.0+)
- Cargo (Rust package manager)
- C compiler (for dependencies)
  - Linux: GCC or Clang
  - Windows: MSVC or MinGW
  - macOS: Xcode Command Line Tools

## Quick Install

### Using Pre-built Binaries

If pre-built binaries are available for your platform:

```bash
# Download the binary
wget https://github.com/ke4ahr/rax25kb/releases/download/v1.5.1/rax25kb-linux-x64

# Make executable
chmod +x rax25kb-linux-x64

# Move to system path
sudo mv rax25kb-linux-x64 /usr/local/bin/rax25kb
```

### Using Installation Script

```bash
# Clone repository
git clone https://github.com/ke4ahr/rax25kb.git
cd rax25kb

# Build and install
./cargo-build.sh
sudo ./cargo-inst.sh
```

This installs:
- Binary: `/usr/local/bin/rax25kb`
- Man page: `/usr/share/man/man1/rax25kb.1`
- Example config: `/etc/rax25kb/rax25kb.cfg.example`

## Building from Source

### Step 1: Install Rust

If you don't have Rust installed:

```bash
# Linux and macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

For Windows, download and run [rustup-init.exe](https://rustup.rs/).

### Step 2: Clone Repository

```bash
git clone https://github.com/ke4ahr/rax25kb.git
cd rax25kb
```

### Step 3: Build

#### Debug Build (for development)
```bash
cargo build
```
Output: `target/debug/rax25kb`

#### Release Build (optimized)
```bash
cargo build --release
```
Output: `target/release/rax25kb`

Or use the provided script:
```bash
./cargo-build.sh
```

### Step 4: Test

```bash
# Show help
./target/release/rax25kb --help

# Test with example config
./target/release/rax25kb -c examples/rax25kb.cfg --dry-run
```

### Step 5: Install

```bash
sudo ./cargo-inst.sh
```

Or manually:
```bash
# Copy binary
sudo cp target/release/rax25kb /usr/local/bin/

# Copy man page
sudo mkdir -p /usr/share/man/man1
sudo cp man/rax25kb-2025-12-20.1 /usr/share/man/man1/rax25kb.1
sudo mandb

# Copy example config
sudo mkdir -p /etc/rax25kb
sudo cp examples/rax25kb.cfg /etc/rax25kb/rax25kb.cfg.example
```

## Platform-Specific Instructions

### Linux (Debian/Ubuntu)

#### Install Build Dependencies
```bash
sudo apt-get update
sudo apt-get install -y build-essential curl pkg-config libudev-dev
```

#### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Build and Install
```bash
git clone https://github.com/ke4ahr/rax25kb.git
cd rax25kb
./cargo-build.sh
sudo ./cargo-inst.sh
```

#### Serial Port Permissions
```bash
# Add user to dialout group
sudo usermod -a -G dialout $USER

# Log out and log back in for changes to take effect
```

#### Verify Installation
```bash
rax25kb --help
man rax25kb
ls -l /dev/ttyUSB* /dev/ttyACM*
```

### Linux (RHEL/CentOS/Fedora)

#### Install Build Dependencies
```bash
sudo dnf install -y gcc make curl pkg-config systemd-devel
# or for CentOS/RHEL 7
sudo yum install -y gcc make curl pkg-config systemd-devel
```

#### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Build and Install
```bash
git clone https://github.com/ke4ahr/rax25kb.git
cd rax25kb
./cargo-build.sh
sudo ./cargo-inst.sh
```

#### Serial Port Permissions
```bash
# Add user to dialout or uucp group
sudo usermod -a -G dialout $USER
# or
sudo usermod -a -G uucp $USER

# Log out and log back in
```

### Windows

#### Install Rust
1. Download [rustup-init.exe](https://rustup.rs/)
2. Run installer and follow prompts
3. Choose "default" installation
4. Restart terminal/PowerShell

#### Install Visual Studio Build Tools (if needed)
1. Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
2. Install "Desktop development with C++"
3. Restart computer

#### Build
```powershell
# Clone repository
git clone https://github.com/ke4ahr/rax25kb.git
cd rax25kb

# Build release version
cargo build --release
```

#### Install
```powershell
# Run as Administrator

# Create installation directory
New-Item -Path "C:\Program Files\rax25kb" -ItemType Directory -Force

# Copy binary
Copy-Item target\release\rax25kb.exe "C:\Program Files\rax25kb\"

# Copy example config
Copy-Item examples\rax25kb.cfg "C:\Program Files\rax25kb\"

# Add to PATH (optional)
$path = [Environment]::GetEnvironmentVariable('Path', 'Machine')
[Environment]::SetEnvironmentVariable('Path', "$path;C:\Program Files\rax25kb", 'Machine')
```

#### Verify Installation
```powershell
# Check version
rax25kb.exe --help

# List COM ports
Get-WmiObject Win32_SerialPort | Select-Object Name,DeviceID
```

### macOS

#### Install Xcode Command Line Tools
```bash
xcode-select --install
```

#### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Build and Install
```bash
git clone https://github.com/ke4ahr/rax25kb.git
cd rax25kb
./cargo-build.sh
sudo ./cargo-inst.sh
```

#### Serial Port Permissions
macOS requires no special permissions for USB serial devices.

#### Verify Installation
```bash
rax25kb --help
ls -l /dev/cu.* /dev/tty.*
```

### FreeBSD

#### Install Dependencies
```bash
sudo pkg install -y rust cargo git
```

#### Build and Install
```bash
git clone https://github.com/ke4ahr/rax25kb.git
cd rax25kb
cargo build --release
sudo cp target/release/rax25kb /usr/local/bin/
sudo cp man/rax25kb-2025-12-20.1 /usr/local/man/man1/rax25kb.1
```

#### Serial Port Permissions
```bash
# Add user to dialer group
sudo pw groupmod dialer -m $USER
```

## Post-Installation

### Create Configuration

```bash
# Generate default configuration
./scripts/generate-default-config.sh rax25kb.cfg

# Or copy example
cp examples/rax25kb.cfg rax25kb.cfg
```

### Edit Configuration

```bash
# Edit for your setup
nano rax25kb.cfg

# At minimum, update:
# - serial_port0000 with your device path
# - cross_connect0000 with desired TCP port
```

### Test Configuration

```bash
# Test without starting
rax25kb -c rax25kb.cfg --dry-run

# Start in foreground (for testing)
rax25kb -c rax25kb.cfg

# Press Ctrl+C to stop
```

### Configure Serial Port

#### Linux
```bash
# Identify your serial device
ls -l /dev/ttyUSB* /dev/ttyACM*

# Update config
serial_port0000=/dev/ttyUSB0
```

#### Windows
```powershell
# List COM ports
Get-WmiObject Win32_SerialPort

# Update config
serial_port0000=COM3
```

#### macOS
```bash
# Identify device
ls -l /dev/cu.* /dev/tty.*

# Update config
serial_port0000=/dev/cu.usbserial-XXXXXXXX
```

### Run as Service

#### Linux (systemd)

Create `/etc/systemd/system/rax25kb.service`:
```ini
[Unit]
Description=rax25kb KISS Bridge
After=network.target

[Service]
Type=simple
User=radio
Group=dialout
ExecStart=/usr/local/bin/rax25kb -c /etc/rax25kb/rax25kb.cfg
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable rax25kb
sudo systemctl start rax25kb
sudo systemctl status rax25kb
```

#### Windows (NSSM)

1. Download NSSM from https://nssm.cc/
2. Install service:
```powershell
nssm install rax25kb "C:\Program Files\rax25kb\rax25kb.exe"
nssm set rax25kb AppParameters -c "C:\Program Files\rax25kb\rax25kb.cfg"
nssm start rax25kb
```

## Troubleshooting

### Build Errors

#### "linker not found"
```bash
# Linux: Install build tools
sudo apt-get install build-essential

# macOS: Install Xcode Command Line Tools
xcode-select --install
```

#### "could not find pkg-config"
```bash
# Linux
sudo apt-get install pkg-config

# macOS
brew install pkg-config
```

#### Rust version too old
```bash
rustup update
```

### Installation Errors

#### Permission denied
```bash
# Run with sudo
sudo ./cargo-inst.sh

# Or manually with sudo
sudo cp target/release/rax25kb /usr/local/bin/
```

#### Serial port not found
```bash
# Linux: Check if device exists
ls -l /dev/ttyUSB* /dev/ttyACM*

# Check dmesg for USB devices
dmesg | grep tty

# Install USB serial drivers if needed
```

#### Permission denied accessing serial port
```bash
# Linux: Add to dialout group
sudo usermod -a -G dialout $USER

# Log out and back in
```

### Runtime Errors

#### "Address already in use"
```bash
# Check what's using the port
sudo netstat -tlnp | grep 8001

# Kill the process or use a different port
```

#### "No such device"
```bash
# Verify serial port path
ls -l /dev/ttyUSB0

# Check if USB device is connected
lsusb
```

## Uninstallation

### Linux/macOS
```bash
# Remove binary
sudo rm /usr/local/bin/rax25kb

# Remove man page
sudo rm /usr/share/man/man1/rax25kb.1
sudo mandb

# Remove configuration (optional)
sudo rm -rf /etc/rax25kb

# Remove user data (optional)
rm -rf ~/.config/rax25kb
```

### Windows
```powershell
# Remove program
Remove-Item "C:\Program Files\rax25kb" -Recurse -Force

# Remove from PATH (if added)
# Use System Properties → Environment Variables → Edit PATH
```

## Getting Help

- **Documentation**: Run `man rax25kb` or see docs/
- **Examples**: See examples/ directory
- **Issues**: https://github.com/ke4ahr/rax25kb/issues

## Next Steps

After installation:
1. Review CROSS-CONNECT-README.md for configuration examples
2. Create your configuration file
3. Test with your TNC
4. Set up as a service for production use
5. Read the documentation for advanced features

## Version Information

This installation guide is for rax25kb version 1.5.1.

For updates and release notes, see CHANGELOG.md.
