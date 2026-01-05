# rax25kb - AX.25 KISS Bridge

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Version](https://img.shields.io/badge/version-1.7.0-green.svg)](https://github.com/ke4ahr/rax25kb/)

**rax25kb** is a multi-port AX.25 KISS bridge that provides flexible connectivity between serial KISS/Extended KISS (XKISS) TNCs and TCP networks, with support for serial-to-serial cross-connections and protocol translation.

**Author:** Kris Kirby, KE4AHR  
**Copyright:** Copyright (C) 2025-2026 Kris Kirby, KE4AHR  
**License:** GPL-3.0-or-later

## Features

- 🔌 **Multi-Port Support**: Up to 10,000 independent cross-connects
- 🔄 **Protocol Translation**: KISS ↔ Extended KISS (XKISS) conversion
- 🌐 **Serial-to-TCP**: Bridge serial TNCs to network applications
- 🔗 **Serial-to-Serial**: Direct TNC-to-TNC connections
- 🛠️ **PhilFlag Correction**: Fixes TASCO modem chipset bugs
- 📦 **PCAP Capture**: Record AX.25 frames for analysis
- 📊 **Frame Parsing**: Display KISS and AX.25 information
- 🔍 **Flexible Logging**: Multi-level logging to console and/or file
- 🖥️ **Cross-Platform**: Linux, Windows, macOS support

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/ke4ahr/rax25kb.git
cd rax25kb

# Build
cargo build --release

# Install (Linux/macOS)
sudo install -m 755 target/release/rax25kb /usr/local/bin/
```

See [INSTALL.md](INSTALL.md) for detailed installation instructions.

### Basic Usage

```bash
# Simple serial to TCP bridge
rax25kb -D /dev/ttyUSB0 -b 9600 -p 8001

# Windows
rax25kb -D COM3 -b 9600 -p 8001

# With configuration file
rax25kb -c /etc/rax25kb/rax25kb.cfg
```

### Example Configuration

```ini
# Single TNC bridge
cross_connect0000.serial_port=/dev/ttyUSB0
cross_connect0000.baud_rate=9600
cross_connect0000.tcp_address=0.0.0.0
cross_connect0000.tcp_port=8001
cross_connect0000.phil_flag=yes

log_level=5
logfile=/var/log/rax25kb.log
```

## Architecture

```
┌─────────────────────────────────────────┐
│          rax25kb v1.6.5                 │
│                                         │
│  ┌─────────────┐    ┌─────────────┐     │
│  │Cross-Connect│    │Cross-Connect│     │
│  │    0000     │    │    0001     │     │
│  │             │    │             │     │
│  │  Serial     │    │  Serial     │     │
│  │    ↕        │    │    ↕        │     │
│  │   TCP       │    │   Serial    │     │
│  └─────────────┘    └─────────────┘     │
│                                         │
│      Logger, PCAP, Translation          │
└─────────────────────────────────────────┘
```

## Use Cases

### 1. Traditional KISS Bridge
Connect a KISS TNC to the network:
```
[KISS TNC] ←→ [rax25kb] ←→ [TCP Application]
```

### 2. Multi-Port Gateway
Bridge multiple TNCs:
```
[TNC 1] ←→ [rax25kb:8001]
[TNC 2] ←→ [rax25kb:8002]
[TNC 3] ←→ [rax25kb:8003]
```

### 3. KISS to XKISS Translation
Protocol conversion:
```
[KISS TNC Port 0] ←→ [rax25kb] ←→ [XKISS TNC Port 5]
```

### 4. Serial Multiplexer
Direct TNC connections:
```
[TNC A] ←→ [rax25kb] ←→ [TNC B]
```

## Documentation

- **[INSTALL.md](INSTALL.md)** - Installation guide for all platforms
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture and design
- **[CHANGELOG.md](CHANGELOG.md)** - Version history and changes
- **[TREE.md](TREE.md)** - Project file structure
- **Man Pages**: `man rax25kb`, `man rax25kb.cfg`
- **Examples**: See `doc/examples/` for configuration examples

### Online Documentation

Full documentation available at: https://github.com/ke4ahr/rax25kb/

## Configuration

### Cross-Connect Configuration

Each cross-connect is an independent bridge with its own settings:

```ini
# Bridge 0: Serial to TCP
cross_connect0000.serial_port=/dev/ttyUSB0
cross_connect0000.baud_rate=9600
cross_connect0000.tcp_address=0.0.0.0
cross_connect0000.tcp_port=8001
cross_connect0000.kiss_port=0

# Bridge 1: Serial to Serial with XKISS
cross_connect0001.serial_port=/dev/ttyUSB1
cross_connect0001.baud_rate=9600
cross_connect0001.xkiss_mode=yes
cross_connect0001.xkiss_port=5
cross_connect0001.serial_to_serial=/dev/ttyUSB0
```

See `man rax25kb.cfg` or `doc/examples/` for more examples.

## Command-Line Options

```
Usage: rax25kb [OPTIONS]

Serial Port:
  -D, --device <dev>      Serial port device
  -b, --baud-rate <rate>  Baud rate (default: 9600)
  -s, --stop-bits <1|2>   Stop bits (default: 1)
  -Q, --parity <n|e|o>    Parity (default: none)

Network:
  -I, --address <addr>    TCP bind address
  -p, --port <port>       TCP port (default: 8001)

Features:
  -n, --phil              Enable PhilFlag correction
  -k, --kiss              Parse KISS frames
  -d, --dump              Dump frames in hex
  -R, --raw-copy          Raw transparent mode

Logging:
  -l, --logfile <file>    Log file path
  -L, --log-level <0-9>   Log level (default: 5)

Other:
  -c <file>               Configuration file
  -h, --help              Show help
```

## Platform Support

### Linux
- Debian/Ubuntu, Fedora, Arch Linux, and derivatives
- systemd service support
- Standard /dev/ttyUSB*, /dev/ttyACM* devices

### Windows
- Windows 10 and later
- COM port support (COM1, COM3, etc.)
- Windows service support via NSSM

### macOS
- macOS 10.15 and later
- /dev/cu.* devices
- launchd service support

## Requirements

- **Rust**: 1.70.0 or later
- **Serial Port**: KISS or XKISS compatible TNC
- **Operating System**: Linux, Windows 10+, or macOS 10.15+

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```

## License

Copyright © 2025-2026 Kris Kirby, KE4AHR

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for guidelines.

## Support

- **Issues**: https://github.com/ke4ahr/rax25kb/issues
- **Documentation**: https://github.com/ke4ahr/rax25kb/
- **Source Code**: https://github.com/ke4ahr/rax25kb/

## Acknowledgments

Special thanks to:
- The amateur radio community
- TAPR (Tucson Amateur Packet Radio)
- Contributors and testers

## Related Projects

- **Direwolf**: Software TNC/APRS decoder
- **soundmodem**: Software modem for packet radio
- **LinBPQ**: BBS and node software
- **AGWPE**: Packet engine for Windows

## Version

Current version: **1.7.3**

See [CHANGELOG.md](CHANGELOG.md) for version history and changes.

---

**73 de KE4AHR**
