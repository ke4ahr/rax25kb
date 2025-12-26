# Migration Guide: rax25kb v1.x to v2.0

## Overview

Version 2.0 introduces multi-port cross-connect support, which changes the configuration file format. This guide helps you migrate your existing configuration.

## Key Changes

### Configuration Structure

**v1.x**: Single serial port, single TCP connection
**v2.0**: Multiple serial ports, multiple cross-connects

### KISS Port Addressing

**Important**: 
- KISS port numbers: 0-15 (default is 0)
- Extended KISS port numbers: 0-15 (default is 0)
- **The default KISS TNC address is always port 0**
- Most TNCs only support port 0

## Quick Migration

### Step 1: Identify Your Current Settings

Look at your old `rax25kb.cfg`:

```ini
# Old v1.x format
serial_port=/dev/ttyUSB0
baud_rate=9600
flow_control=none
stop_bits=1
parity=none
tcp_address=0.0.0.0
tcp_port=8001
phil_flag=false
parse_kiss=true
```

### Step 2: Convert to v2.0 Format

Create new configuration with these changes:

1. **Add serial port ID** (use 0000 for first port)
2. **Create cross-connect** linking serial and TCP
3. **Move per-connection settings** to cross-connect

```ini
# New v2.0 format

# Serial port definition (add ID 0000)
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600
serial_port0000_flow_control=none
serial_port0000_stop_bits=1
serial_port0000_parity=none
serial_port0000_extended_kiss=false

# Cross-connect definition (KISS port 0 is default)
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
cross_connect0000_phil_flag=false
cross_connect0000_parse_kiss=true

# Global settings (unchanged)
log_level=5
```

### Step 3: Note the Defaults

If you didn't specify these in v1.x, they're still the defaults:

```ini
# These are default values, no need to specify unless changing
serial_port0000_baud=9600
serial_port0000_flow_control=none
serial_port0000_stop_bits=1
serial_port0000_parity=none
serial_port0000_extended_kiss=false
```

## Migration Examples

### Example 1: Basic Configuration

**Old (v1.x)**:
```ini
serial_port=/dev/ttyUSB0
baud_rate=9600
tcp_port=8001
parse_kiss=true
log_level=5
```

**New (v2.0)**:
```ini
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
cross_connect0000_parse_kiss=true
log_level=5
```

### Example 2: With PhilFlag

**Old (v1.x)**:
```ini
serial_port=/dev/ttyUSB0
baud_rate=9600
tcp_port=8001
phil_flag=true
parse_kiss=true
```

**New (v2.0)**:
```ini
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
cross_connect0000_phil_flag=true
cross_connect0000_parse_kiss=true
```

### Example 3: Hardware Flow Control

**Old (v1.x)**:
```ini
serial_port=/dev/ttyUSB0
baud_rate=115200
flow_control=hardware
tcp_port=8001
```

**New (v2.0)**:
```ini
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=115200
serial_port0000_flow_control=hardware
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
```

### Example 4: Kenwood TS-2000

**Old (v1.x)**:
```ini
serial_port=/dev/ttyUSB0
baud_rate=4800
stop_bits=2
phil_flag=true
tcp_port=8001
parse_kiss=true
```

**New (v2.0)** - Recommended KISS settings for TS-2000:
```ini
# Kenwood TS-2000 Internal TNC
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=4800
serial_port0000_stop_bits=2
serial_port0000_parity=none
serial_port0000_flow_control=none

# KISS port 0 (default TNC address)
cross_connect0000=serial:0000:0 <-> tcp:127.0.0.1:8001
cross_connect0000_phil_flag=true
cross_connect0000_parse_kiss=true
```

### Example 5: With Logging

**Old (v1.x)**:
```ini
serial_port=/dev/ttyUSB0
baud_rate=9600
tcp_port=8001
parse_kiss=true
log_level=6
logfile=/var/log/rax25kb.log
pidfile=/var/run/rax25kb.pid
```

**New (v2.0)**:
```ini
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
cross_connect0000_parse_kiss=true
log_level=6
logfile=/var/log/rax25kb.log
pidfile=/var/run/rax25kb.pid
```

### Example 6: Windows

**Old (v1.x)**:
```ini
serial_port=COM3
baud_rate=9600
tcp_port=8001
logfile=C:\ProgramData\rax25kb\rax25kb.log
```

**New (v2.0)**:
```ini
serial_port0000=COM3
serial_port0000_baud=9600
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
logfile=C:\ProgramData\rax25kb\rax25kb.log
```

## Setting Mapping Table

| Old Setting (v1.x) | New Setting (v2.0) |
|--------------------|-------------------|
| `serial_port` | `serial_port0000` |
| `baud_rate` | `serial_port0000_baud` |
| `flow_control` | `serial_port0000_flow_control` |
| `stop_bits` | `serial_port0000_stop_bits` |
| `parity` | `serial_port0000_parity` |
| `tcp_address` + `tcp_port` | `tcp:address:port` in cross-connect |
| `phil_flag` | `cross_connect0000_phil_flag` |
| `parse_kiss` | `cross_connect0000_parse_kiss` |
| `dump_frames` | `cross_connect0000_dump` |
| `dump_ax25` | `cross_connect0000_dump_ax25` |
| `raw_copy` | `cross_connect0000_raw_copy` |
| `log_level` | `log_level` (unchanged) |
| `logfile` | `logfile` (unchanged) |
| `pidfile` | `pidfile` (unchanged) |
| `pcap_file` | `pcap_file` (unchanged) |

## New Capabilities in v2.0

After migrating, you can now:

### Add More TNCs

```ini
# First TNC
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001

# Second TNC
serial_port0001=/dev/ttyUSB1
serial_port0001_baud=9600
cross_connect0001=serial:0001:0 <-> tcp:0.0.0.0:8002

# Third TNC
serial_port0002=/dev/ttyUSB2
serial_port0002_baud=1200
cross_connect0002=serial:0002:0 <-> tcp:0.0.0.0:8003
```

### Bridge TNCs Together

```ini
serial_port0000=/dev/ttyUSB0
serial_port0001=/dev/ttyUSB1

# Bridge for digipeating
cross_connect0000=serial:0000:0 <-> serial:0001:0
```

### Use Multiple KISS Ports

**Note**: Only if your TNC supports multiple ports!

```ini
serial_port0000=/dev/ttyUSB0

# KISS port 0 (default)
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001

# KISS port 1
cross_connect0001=serial:0000:1 <-> tcp:0.0.0.0:8002
```

## Testing Your Migration

### 1. Backup Old Configuration

```bash
cp rax25kb.cfg rax25kb.cfg.v1.backup
```

### 2. Create New Configuration

Create your new v2.0 configuration file.

### 3. Test Syntax

```bash
rax25kb -c rax25kb.cfg -h
```

If no errors, the config is valid.

### 4. Test Connection

```bash
# Start rax25kb
rax25kb -c rax25kb.cfg

# In another terminal, test connection
telnet localhost 8001
```

### 5. Verify KISS Frames

Enable parsing to verify data flow:

```ini
cross_connect0000_parse_kiss=true
```

You should see KISS frame information in the output.

## Common Migration Issues

### Issue: "Missing required config: serial_port"

**Cause**: Using old v1.x config file with v2.0

**Solution**: Convert to new format with `serial_port0000`

### Issue: "Unknown serial port ID: 0000"

**Cause**: Cross-connect defined before serial port

**Solution**: Define serial port before cross-connect:
```ini
serial_port0000=/dev/ttyUSB0  # First
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001  # Second
```

### Issue: "KISS port must be 0-15"

**Cause**: Invalid KISS port number

**Solution**: Use port 0 (default) unless your TNC supports multiple ports:
```ini
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
```

### Issue: No packets received after migration

**Cause**: Using wrong KISS port number

**Solution**: Most TNCs only support port 0 (default):
```ini
# Use port 0 (default TNC address)
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
```

## Getting Help

If you have issues migrating:

1. Check the example configuration file
2. Review the CROSS-CONNECT-GUIDE.md
3. Open an issue on GitHub: https://github.com/ke4ahr/rax25kb/issues

Include:
- Your old v1.x configuration
- Your new v2.0 configuration
- Error messages
- TNC make/model