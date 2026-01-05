# Cross-Connect Guide for rax25kb v1.6.7

**Copyright (C) 2025-2026 Kris Kirby, KE4AHR**  
**SPDX-License-Identifier: GPL-3.0-or-later**

## Table of Contents

1. [Introduction](#introduction)
2. [Cross-Connect Basics](#cross-connect-basics)
3. [Connection Types](#connection-types)
4. [Configuration Reference](#configuration-reference)
5. [Advanced Features](#advanced-features)
6. [Security Considerations](#security-considerations)
7. [Examples](#examples)
8. [Troubleshooting](#troubleshooting)

---

## Introduction

Cross-connects are the fundamental building blocks of rax25kb. Each cross-connect represents an independent data bridge with its own configuration, threading, and behavior. This guide explains how cross-connects work and how to configure them for various scenarios.

### What is a Cross-Connect?

A **cross-connect** bridges data between two endpoints:
- **Serial port** ↔ **TCP server** (traditional)
- **Serial port** ↔ **Serial port** (with protocol translation)
- **Serial port** ↔ **TCP client** (NEW in v1.6.7)
- **TCP client** ↔ **TCP server** (DANGEROUS, requires flags)

### Why Cross-Connects?

- **Independence**: Each operates separately with own threads
- **Flexibility**: Mix serial, TCP server, TCP client in one instance
- **Scalability**: Up to 10,000 cross-connects (0000-9999)
- **Protocol Translation**: KISS ↔ XKISS between endpoints

---

## Cross-Connect Basics

### Naming Convention

Cross-connects are numbered from **0000 to 9999**:
```ini
cross_connect0000.parameter=value
cross_connect0001.parameter=value
cross_connect9999.parameter=value
```

### Core Components

Every cross-connect has:

1. **Unique ID**: `cross_connectXXXX`
2. **Source Endpoint**: Usually a serial port
3. **Destination Endpoint**: TCP server, TCP client, or another serial port
4. **Processing Options**: KISS/XKISS, PhilFlag, parsing, buffering
5. **Independent Threads**: Accept, read, write threads per connection

### Threading Model

Each cross-connect spawns its own threads:

```
Cross-Connect 0000
├── TCP Accept Thread (if server mode)
├── TCP Client Thread (if client mode)
├── TCP → Serial Thread (per client)
└── Serial → TCP Thread (continuous)

Cross-Connect 0001
├── (independent threads)
└── ...
```

Threads use `Arc<Mutex<>>` for safe shared access to serial ports and network connections.

---

## Connection Types

### 1. Serial-to-TCP Server (Traditional)

**Use Case**: Provide network access to a serial KISS TNC

```
[Serial TNC] ←─serial─→ [rax25kb] ←─TCP Server─→ [Applications]
```

**Configuration:**
```ini
cross_connect0000.serial_port=/dev/ttyUSB0
cross_connect0000.baud_rate=9600
cross_connect0000.tcp_mode=server          # or omit (default)
cross_connect0000.tcp_address=0.0.0.0
cross_connect0000.tcp_port=8001
cross_connect0000.kiss_port=0
```

**Characteristics:**
- ✅ **Single client**: Only one TCP connection at a time
- ✅ **Bidirectional**: Data flows both ways
- ✅ **KISS processing**: Frame parsing, PhilFlag optional
- ✅ **Most common**: Standard use case

---

### 2. Serial-to-TCP Client (NEW in v1.6.7)

**Use Case**: Connect to a remote TCP server from serial TNC

```
[Serial TNC] ←─serial─→ [rax25kb] ←─TCP Client─→ [Remote TCP Server]
```

**Configuration:**
```ini
cross_connect0001.serial_port=/dev/ttyUSB1
cross_connect0001.baud_rate=9600
cross_connect0001.tcp_mode=client
cross_connect0001.tcp_server_address=192.168.1.100
cross_connect0001.tcp_server_port=8001
cross_connect0001.kiss_port=0
```

**Characteristics:**
- ✅ **Auto-reconnect**: Attempts reconnection on disconnect
- ✅ **Outbound**: rax25kb initiates connection
- ✅ **Useful for**: Connecting to remote bridges, gateways
- ⚠️ **Not yet implemented**: Full TCP client logic pending

---

### 3. Serial-to-Serial (Protocol Translation)

**Use Case**: Connect two serial TNCs with KISS/XKISS translation

```
[KISS TNC Port 0] ←→ [rax25kb] ←→ [XKISS TNC Port 5]
```

**Configuration:**
```ini
# KISS Side
cross_connect0002.serial_port=/dev/ttyUSB2
cross_connect0002.baud_rate=9600
cross_connect0002.kiss_port=0
cross_connect0002.serial_to_serial=/dev/ttyUSB3

# XKISS Side
cross_connect0003.serial_port=/dev/ttyUSB3
cross_connect0003.baud_rate=9600
cross_connect0003.xkiss_mode=yes
cross_connect0003.xkiss_port=5
```

**Characteristics:**
- ✅ **Direct connection**: No TCP involved
- ✅ **Translation**: Automatic port number translation
- ✅ **Bidirectional**: Data flows both ways
- ⚠️ **Both sides needed**: Requires two cross-connects

---

### 4. TCP-to-TCP (DANGEROUS)

**Use Case**: Bridge between two TCP connections (**NOT RECOMMENDED**)

```
[TCP Server] ←→ [rax25kb] ←→ [TCP Client]
```

**Configuration:**
```ini
# Server Side (receiving)
cross_connect0004.tcp_mode=server
cross_connect0004.tcp_address=0.0.0.0
cross_connect0004.tcp_port=8004
cross_connect0004.tcp_to_tcp_dangerous=true          # REQUIRED
cross_connect0004.tcp_to_tcp_also_dangerous=false    # Enforce KISS

# Client Side (sending)
cross_connect0005.tcp_mode=client
cross_connect0005.tcp_server_address=192.168.1.200
cross_connect0005.tcp_server_port=8005
cross_connect0005.tcp_to_tcp_dangerous=true
```

**⚠️ WARNING - DANGEROUS CONFIGURATION**

TCP-to-TCP bypasses normal serial port safety:
- ❌ **Security risk**: External systems can inject packets
- ❌ **Data corruption**: Format mismatches cause corruption
- ❌ **Network loops**: Misconfiguration creates infinite loops
- ❌ **No isolation**: Bypasses hardware serial port protection

**Safety Flags:**
- `tcp_to_tcp_dangerous=true`: **Required** to enable TCP-to-TCP
- `tcp_to_tcp_also_dangerous=false`: Enforce KISS packet validation
- `tcp_to_tcp_also_dangerous=true`: Allow **any** data (VERY RISKY)

**When to Use:**
- ✅ Controlled lab environment
- ✅ Trusted network only
- ✅ Understand all risks
- ❌ Never in production
- ❌ Never on internet-facing systems

**Packet Validation:**
When `tcp_to_tcp_also_dangerous=false` (default):
- Only KISS-framed packets allowed (`[FEND]...[FEND]`)
- Non-KISS packets rejected and logged
- Provides minimal safety check

When `tcp_to_tcp_also_dangerous=true`:
- **NO validation** - all data passes through
- Maximum danger
- Only use if you absolutely know what you're doing

---

## Configuration Reference

### Required Parameters

Every cross-connect needs:
```ini
cross_connectXXXX.serial_port=/dev/ttyUSB0  # Or COM3 on Windows
```

### TCP Server Parameters
```ini
cross_connectXXXX.tcp_mode=server           # Default if omitted
cross_connectXXXX.tcp_address=0.0.0.0       # Bind address
cross_connectXXXX.tcp_port=8001             # Listen port
```

### TCP Client Parameters (NEW)
```ini
cross_connectXXXX.tcp_mode=client
cross_connectXXXX.tcp_server_address=host   # Remote server
cross_connectXXXX.tcp_server_port=port      # Remote port
```

### Serial Port Parameters
```ini
cross_connectXXXX.baud_rate=9600            # Default: 9600
cross_connectXXXX.data_bits=8               # 7 or 8, default: 8
cross_connectXXXX.stop_bits=1               # 1 or 2, default: 1
cross_connectXXXX.parity=none               # none, even, odd
cross_connectXXXX.flow_control=none         # none, software, hardware, dtrdsr
```

**Note**: When KISS/XKISS is active, serial forced to 8N1 per spec

### KISS Parameters
```ini
cross_connectXXXX.kiss_port=0               # KISS port number (0-15)
cross_connectXXXX.phil_flag=no              # PhilFlag correction
cross_connectXXXX.parse_kiss=no             # Parse and log frames
cross_connectXXXX.dump=no                   # Hex dump
cross_connectXXXX.raw_copy=no               # Transparent mode
```

### XKISS Parameters
```ini
cross_connectXXXX.xkiss_mode=no             # Enable XKISS
cross_connectXXXX.xkiss_port=5              # XKISS port number
cross_connectXXXX.xkiss_checksum=no         # Add/verify checksums
cross_connectXXXX.xkiss_polling=no          # Polling mode
cross_connectXXXX.xkiss_poll_timer_ms=100   # Poll interval
cross_connectXXXX.xkiss_rx_buffer_size=16384  # RX buffer (4KB-1MB)
```

### TCP-to-TCP Parameters (DANGEROUS)
```ini
cross_connectXXXX.tcp_to_tcp_dangerous=false       # Enable TCP-to-TCP
cross_connectXXXX.tcp_to_tcp_also_dangerous=false  # Disable validation
```

### Packet Reframing (NEW)
```ini
cross_connectXXXX.reframe_large_packets=no  # Auto-split large packets
```

---

## Advanced Features

### XKISS RX Buffer (NEW in v1.6.7)

**Purpose**: Buffer received packets when polling mode is active

**Configuration:**
```ini
cross_connectXXXX.xkiss_polling=yes
cross_connectXXXX.xkiss_rx_buffer_size=16384  # 16KB default
```

**How it Works:**
1. Packets arrive from serial port
2. Stored in buffer (up to buffer_size bytes)
3. On poll cycle, buffer flushed to destination
4. If buffer full, packets dropped (logged)

**Buffer Size Limits:**
- **Minimum**: 4096 bytes (4 KB)
- **Default**: 16384 bytes (16 KB)
- **Maximum**: 1048576 bytes (1 MB)

**When to Use:**
- ✅ XKISS polling mode enabled
- ✅ Bursty traffic patterns
- ✅ Need to accumulate packets between polls

**When Not to Use:**
- ❌ Standard KISS (not needed)
- ❌ Low memory systems
- ❌ Real-time applications (adds latency)

---

### Packet Reframing (NEW in v1.6.7)

**Purpose**: Automatically split large packets that exceed 255 bytes after PhilFlag processing

**Configuration:**
```ini
cross_connectXXXX.phil_flag=yes
cross_connectXXXX.reframe_large_packets=yes
```

**How it Works:**
1. Detect if PhilFlag translation will exceed 255 bytes
2. Parse AX.25 packet structure
3. Split info field into chunks
4. Wrap each chunk in KISS framing
5. Send fragments sequentially

**Limitations:**
- Only splits info field (not headers)
- No reassembly on receiving end
- May confuse some applications
- Conservative chunk size (220 bytes)

**When to Use:**
- ✅ Known large packets
- ✅ PhilFlag enabled (creates expansion)
- ✅ Receiving software can handle fragments

**When Not to Use:**
- ❌ Most cases (packets rarely exceed 255 bytes)
- ❌ Legacy software expecting complete packets
- ❌ Unknown receiving software capabilities

---

## Security Considerations

### TCP-to-TCP Dangers

**Why is TCP-to-TCP dangerous?**

1. **Bypasses Hardware Isolation**
   - Serial ports provide electrical isolation
   - TCP-to-TCP removes this protection
   - One compromised system affects all

2. **Packet Injection**
   - External systems can inject arbitrary packets
   - No hardware rate limiting
   - Amplification attacks possible

3. **Data Corruption**
   - Different applications expect different formats
   - Format mismatches cause silent corruption
   - Debug is extremely difficult

4. **Network Loops**
   - Easy to create routing loops
   - Can saturate network
   - Difficult to detect

5. **No Authentication**
   - Anyone with network access can connect
   - No encryption
   - No access control

**Mitigation Strategies:**

1. **Use Firewall Rules**
```bash
# Only allow from specific hosts
iptables -A INPUT -p tcp --dport 8004 -s 192.168.1.100 -j ACCEPT
iptables -A INPUT -p tcp --dport 8004 -j DROP
```

2. **Enforce KISS Validation**
```ini
tcp_to_tcp_dangerous=true
tcp_to_tcp_also_dangerous=false  # Keep this false!
```

3. **Monitor Logs**
```bash
tail -f /var/log/rax25kb.log | grep "TCP-to-TCP"
```

4. **Use Separate Network**
- Dedicated VLAN
- Air-gapped network
- VPN tunnel

5. **Test Thoroughly**
- Lab environment first
- Monitor for loops
- Have kill switch ready

---

### Serial Port Security

**Less Dangerous** but still important:

1. **Physical Access**: Serial ports require physical access
2. **Electrical Isolation**: Hardware provides isolation
3. **Rate Limiting**: Serial bandwidth naturally limited
4. **Known Format**: KISS format well-defined

**Best Practices:**
- Lock server room
- Monitor serial port access
- Use hardware flow control when possible
- Enable PhilFlag only if needed

---

## Examples

### Example 1: Simple TNC Gateway
```ini
# Single TNC on TCP port 8001
cross_connect0000.serial_port=/dev/ttyUSB0
cross_connect0000.baud_rate=9600
cross_connect0000.tcp_address=0.0.0.0
cross_connect0000.tcp_port=8001
cross_connect0000.kiss_port=0
cross_connect0000.phil_flag=yes
```

### Example 2: Multiple TNCs
```ini
# VHF TNC
cross_connect0000.serial_port=/dev/ttyUSB0
cross_connect0000.baud_rate=9600
cross_connect0000.tcp_port=8001

# UHF TNC
cross_connect0001.serial_port=/dev/ttyUSB1
cross_connect0001.baud_rate=9600
cross_connect0001.tcp_port=8002

# HF TNC
cross_connect0002.serial_port=/dev/ttyUSB2
cross_connect0002.baud_rate=19200
cross_connect0002.tcp_port=8003
```

### Example 3: KISS to XKISS Translation
```ini
# KISS TNC (port 0)
cross_connect0000.serial_port=/dev/ttyUSB0
cross_connect0000.baud_rate=9600
cross_connect0000.kiss_port=0
cross_connect0000.serial_to_serial=/dev/ttyUSB1

# XKISS TNC (port 5)
cross_connect0001.serial_port=/dev/ttyUSB1
cross_connect0001.baud_rate=9600
cross_connect0001.xkiss_mode=yes
cross_connect0001.xkiss_port=5
cross_connect0001.xkiss_checksum=yes
```

### Example 4: TCP Client Mode
```ini
# Connect to remote server
cross_connect0002.serial_port=/dev/ttyUSB2
cross_connect0002.baud_rate=9600
cross_connect0002.tcp_mode=client
cross_connect0002.tcp_server_address=192.168.1.100
cross_connect0002.tcp_server_port=8001
```

### Example 5: XKISS with Buffering
```ini
cross_connect0003.serial_port=/dev/ttyUSB3
cross_connect0003.baud_rate=9600
cross_connect0003.xkiss_mode=yes
cross_connect0003.xkiss_port=3
cross_connect0003.xkiss_polling=yes
cross_connect0003.xkiss_poll_timer_ms=100
cross_connect0003.xkiss_rx_buffer_size=32768  # 32KB
cross_connect0003.tcp_port=8003
```

### Example 6: TCP-to-TCP (Lab Only!)
```ini
# WARNING: Dangerous configuration - lab use only!

# Server side
cross_connect0004.tcp_mode=server
cross_connect0004.tcp_address=127.0.0.1  # Localhost only!
cross_connect0004.tcp_port=8004
cross_connect0004.tcp_to_tcp_dangerous=true
cross_connect0004.tcp_to_tcp_also_dangerous=false

# Client side
cross_connect0005.tcp_mode=client
cross_connect0005.tcp_server_address=127.0.0.1
cross_connect0005.tcp_server_port=8005
cross_connect0005.tcp_to_tcp_dangerous=true
```

---

## Troubleshooting

### Common Issues

#### 1. "Connection refused" on TCP port
**Cause**: Port already in use or permission denied

**Solution:**
```bash
# Check if port is in use
netstat -tulpn | grep 8001

# Check permissions
sudo rax25kb -c config.cfg  # Run as root if needed
```

#### 2. "Serial port access denied"
**Cause**: User not in dialout group

**Solution:**
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

#### 3. "Buffer full" errors
**Cause**: XKISS RX buffer overflow

**Solution:**
```ini
# Increase buffer size
xkiss_rx_buffer_size=65536  # 64KB

# Or decrease poll timer
xkiss_poll_timer_ms=50  # More frequent polls
```

#### 4. "TCP-to-TCP dangerous flag required"
**Cause**: Attempting TCP-to-TCP without flag

**Solution:**
```ini
# Enable dangerous flag (understand risks first!)
tcp_to_tcp_dangerous=true
```

#### 5. Packets corrupted or lost
**Possible causes:**
- PhilFlag needed but not enabled
- Wrong serial port settings
- TCP-to-TCP without KISS validation

**Solution:**
```ini
# Enable PhilFlag for TASCO modems
phil_flag=yes

# Verify serial settings
baud_rate=9600
data_bits=8  # Will be forced for KISS anyway

# Enable KISS validation for TCP-to-TCP
tcp_to_tcp_also_dangerous=false
```

#### 6. Large packets not arriving
**Cause**: Exceed 255 bytes after PhilFlag

**Solution:**
```ini
# Enable automatic reframing
reframe_large_packets=yes
```

---

## Performance Considerations

### Buffer Sizes
- **4 KB**: Minimal memory, may drop packets under load
- **16 KB**: Good default for most uses
- **64 KB**: High-traffic scenarios
- **1 MB**: Extreme buffering (rarely needed)

### Poll Timers
- **50 ms**: Low latency, higher CPU usage
- **100 ms**: Good default balance
- **250 ms**: Low CPU, higher latency
- **1000 ms**: Very low CPU, significant latency

### Thread Count
Each cross-connect creates 2-4 threads:
- 10 cross-connects = ~30 threads
- 100 cross-connects = ~300 threads
- Monitor with: `ps -eLf | grep rax25kb | wc -l`

### CPU Usage
- Serial I/O: Low impact
- KISS parsing: Very low impact
- PhilFlag: Minimal impact
- Packet reframing: Moderate impact (rare)
- TCP-to-TCP: Low impact but risky

---

## Best Practices

1. **Start Simple**: One cross-connect, verify it works
2. **Add Gradually**: Add cross-connects one at a time
3. **Test Thoroughly**: Lab environment before production
4. **Monitor Logs**: Watch for errors and warnings
5. **Document Config**: Comment your configuration file
6. **Avoid TCP-to-TCP**: Unless absolutely necessary
7. **Use Default Buffers**: 16KB is usually sufficient
8. **Enable Validation**: Keep `tcp_to_tcp_also_dangerous=false`
9. **Firewall Rules**: Protect TCP ports
10. **Regular Updates**: Keep rax25kb up to date

---

## Conclusion

Cross-connects provide powerful flexibility but require careful configuration. Start with simple serial-to-TCP setups, understand the basics, then explore advanced features as needed. Always prioritize security, especially with TCP-to-TCP connections.

For more information:
- Main documentation: `man rax25kb`
- Configuration reference: `man rax25kb.cfg`
- Project website: https://github.com/ke4ahr/rax25kb/

---

**Version**: 1.6.7  
**Last Updated**: 2025-12-30  
**Author**: Kris Kirby, KE4AHR  
**License**: GPL-3.0-or-later
