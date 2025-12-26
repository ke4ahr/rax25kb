# rax25kb Cross-Connect Configuration Guide

## Overview

rax25kb version 2.0 introduces support for multiple serial ports and flexible cross-connect configurations. This allows you to:

- Connect multiple TNCs simultaneously
- Route KISS ports to different TCP ports
- Bridge serial ports together
- Translate between standard KISS and Extended KISS
- Configure each connection independently

## KISS Port Addressing

### Important Notes

- **Standard KISS port numbers**: 0-15 (4-bit addressing)
- **Extended KISS port numbers**: 0-15 (4-bit addressing)
- **Default KISS TNC address**: Always port 0
- **Port 0**: The standard default for all TNCs

Most TNCs only support port 0. Multi-port support requires:
- TNC hardware/firmware that supports multiple ports
- Proper TNC configuration to enable additional ports
- Consult your TNC manual for multi-port capabilities

## Configuration File Format

### Serial Port Definitions

Each serial port is defined with a unique 4-digit ID (0000-9999):

```ini
serial_portXXXX=/dev/ttyUSB0
serial_portXXXX_baud=9600
serial_portXXXX_flow_control=none|software|hardware|dtrdsr
serial_portXXXX_stop_bits=1|2
serial_portXXXX_parity=none|even|odd
serial_portXXXX_extended_kiss=false|true
```

**Required**:
- `serial_portXXXX`: Device path (Linux: `/dev/ttyUSB0`, Windows: `COM3`)

**Optional** (with defaults):
- `serial_portXXXX_baud`: Baud rate (default: 9600)
- `serial_portXXXX_flow_control`: Flow control (default: none)
- `serial_portXXXX_stop_bits`: Stop bits (default: 1)
- `serial_portXXXX_parity`: Parity (default: none)
- `serial_portXXXX_extended_kiss`: Extended KISS support (default: false)

### Cross-Connect Definitions

Each cross-connect links two endpoints:

```ini
cross_connectXXXX=endpoint_a <-> endpoint_b
```

**Endpoint Types**:
- `tcp:address:port` - TCP socket listener
- `serial:port_id:kiss_port` - Serial port with KISS port number (0-15)

**Optional Settings**:
```ini
cross_connectXXXX_phil_flag=false|true
cross_connectXXXX_dump=false|true
cross_connectXXXX_parse_kiss=false|true
cross_connectXXXX_dump_ax25=false|true
cross_connectXXXX_raw_copy=false|true
```

## Common Serial Formats

### 8N1 (Most Common)
- 8 data bits, No parity, 1 stop bit
- Standard for most KISS TNCs
- Default in rax25kb

### 8N2 (Some TNCs)
- 8 data bits, No parity, 2 stop bits
- Required by some TNCs (e.g., Kenwood TS-2000)

### 8E1 (Rare)
- 8 data bits, Even parity, 1 stop bit
- Uncommon in amateur radio

## Configuration Examples

### Example 1: Simple Single TNC

The most basic configuration - one TNC on port 0:

```ini
# Serial port definition
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600

# Cross-connect: TNC KISS port 0 (default) to TCP
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
cross_connect0000_parse_kiss=true

# Logging
log_level=5
```

**What this does**:
- Opens `/dev/ttyUSB0` at 9600 baud (8N1 default)
- Connects KISS port 0 (default TNC address) to TCP port 8001
- Enables KISS frame parsing

### Example 2: Kenwood TS-2000 (Recommended Settings)

The TS-2000 requires specific serial settings:

```ini
# TS-2000 Internal TNC
# Recommended KISS settings for TS-2000
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=4800
serial_port0000_flow_control=none
serial_port0000_stop_bits=2
serial_port0000_parity=none
serial_port0000_extended_kiss=false

# Cross-connect with PhilFlag for proper KISS handling
cross_connect0000=serial:0000:0 <-> tcp:127.0.0.1:8001
cross_connect0000_phil_flag=true
cross_connect0000_parse_kiss=true

log_level=5
```

**Why these settings**:
- **4800 baud**: TS-2000 KISS mode default speed
- **8N2**: Required by TS-2000 (2 stop bits)
- **PhilFlag**: Fixes TS-2000 KISS escaping issues
- **Port 0**: Default KISS TNC address
- **Localhost**: More secure than public access

### Example 3: Multiple TNCs

Connect three different TNCs to three TCP ports:

```ini
# TNC 1 - VHF APRS
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001

# TNC 2 - UHF Messaging
serial_port0001=/dev/ttyUSB1
serial_port0001_baud=9600
cross_connect0001=serial:0001:0 <-> tcp:0.0.0.0:8002

# TNC 3 - HF Packet
serial_port0002=/dev/ttyUSB2
serial_port0002_baud=1200
cross_connect0002=serial:0002:0 <-> tcp:0.0.0.0:8003

log_level=5
```

### Example 4: Multi-Port TNC

If your TNC supports multiple KISS ports:

```ini
# Single TNC with multi-port support
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600

# KISS port 0 - APRS
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001

# KISS port 1 - Messaging
cross_connect0001=serial:0000:1 <-> tcp:0.0.0.0:8002

# KISS port 2 - Telemetry
cross_connect0002=serial:0000:2 <-> tcp:0.0.0.0:8003

log_level=5
```

**Note**: Most TNCs only support port 0. Check your TNC manual!

### Example 5: Serial-to-Serial Bridge

Connect two TNCs together for digipeating:

```ini
# TNC 1
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600

# TNC 2
serial_port0001=/dev/ttyUSB1
serial_port0001_baud=9600

# Bridge them together on KISS port 0 (default)
cross_connect0000=serial:0000:0 <-> serial:0001:0
cross_connect0000_parse_kiss=true

log_level=5
```

### Example 6: KISS Port Translation

Translate between different KISS ports:

```ini
# Standard KISS TNC
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600
serial_port0000_extended_kiss=false

# Extended KISS TNC
serial_port0001=/dev/ttyUSB1
serial_port0001_baud=9600
serial_port0001_extended_kiss=true

# Translate standard KISS port 0 to extended KISS port 3
cross_connect0000=serial:0000:0 <-> serial:0001:3
cross_connect0000_parse_kiss=true

log_level=5
```

### Example 7: Windows Configuration

```ini
# Windows COM ports
serial_port0000=COM3
serial_port0000_baud=9600

serial_port0001=COM4
serial_port0001_baud=115200

# Cross-connects
cross_connect0000=serial:0000:0 <-> tcp:127.0.0.1:8001
cross_connect0001=serial:0001:0 <-> tcp:0.0.0.0:8002

# Windows paths
logfile=C:\ProgramData\rax25kb\rax25kb.log
pidfile=C:\ProgramData\rax25kb\rax25kb.pid

log_level=5
```

## TNC-Specific Settings

### Kenwood TS-2000
```ini
serial_port0000_baud=4800
serial_port0000_stop_bits=2
serial_port0000_parity=none
cross_connect0000_phil_flag=true
```

### Mobilinkd TNC3
```ini
serial_port0000_baud=115200
serial_port0000_flow_control=hardware
serial_port0000_stop_bits=1
```

### Kantronics KPC-3+
```ini
serial_port0000_baud=9600
serial_port0000_stop_bits=1
serial_port0000_parity=none
```

### Argent Data Tracker2
```ini
serial_port0000_baud=9600
serial_port0000_stop_bits=1
serial_port0000_parity=none
```

## Migration from Version 1.x

### Old Configuration (v1.x)
```ini
serial_port=/dev/ttyUSB0
baud_rate=9600
tcp_address=0.0.0.0
tcp_port=8001
phil_flag=false
parse_kiss=true
```

### New Configuration (v2.0)
```ini
# Define serial port
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600

# Define cross-connect (KISS port 0 is default)
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
cross_connect0000_phil_flag=false
cross_connect0000_parse_kiss=true

# Global settings remain similar
log_level=5
```

**Key Changes**:
- Serial ports now have IDs (0000-9999)
- Cross-connects explicitly define connections
- Each cross-connect has its own settings
- Must specify KISS port number (usually 0)

## Troubleshooting

### "Unknown serial port ID"
**Problem**: Cross-connect references undefined serial port

**Solution**: Ensure serial port is defined before cross-connect:
```ini
serial_port0000=/dev/ttyUSB0  # Define first
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001  # Then use
```

### "KISS port must be 0-15"
**Problem**: KISS port number out of range

**Solution**: Use ports 0-15 only:
```ini
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001  # Correct
cross_connect0001=serial:0000:16 <-> tcp:0.0.0.0:8002  # Wrong!
```

### No packets received
**Problem**: Wrong KISS port number

**Solution**: Most TNCs use port 0 (default):
```ini
# Try port 0 first (default)
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
```

### Serial port permission denied
**Linux Solution**:
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

### Packets corrupted
**Problem**: May need PhilFlag for TASCO modems or TS-2000

**Solution**:
```ini
cross_connect0000_phil_flag=true
```

## Best Practices

### Security
- Use `127.0.0.1` for local-only access
- Use `0.0.0.0` only when remote access needed
- Enable firewall rules for public-facing connections

### Performance
- Use hardware flow control for high-speed TNCs (>9600 baud)
- Set appropriate baud rate for your TNC
- Don't enable verbose logging in production

### Reliability
- Always specify KISS port 0 (default) unless you know your TNC supports multiple ports
- Use 8N1 format unless your TNC requires otherwise
- Test with one TNC before adding multiple

### Organization
- Use sequential IDs (0000, 0001, 0002...)
- Comment your configuration file
- Document your KISS port assignments

## Command-Line Usage

```bash
# Use specific configuration file
rax25kb -c myconfig.cfg

# Quiet startup
rax25kb -c myconfig.cfg -q

# Show help
rax25kb -h

# Show version
rax25kb -v
```

## Logging

Set log level (0-9):
```ini
log_level=5  # NOTICE (default)
log_level=6  # INFO
log_level=7  # DEBUG
```

## Support

For more information:
- GitHub: https://github.com/ke4ahr/rax25kb
- Issues: https://github.com/ke4ahr/rax25kb/issues
