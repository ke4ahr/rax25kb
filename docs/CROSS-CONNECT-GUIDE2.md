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
- **Default KISS TNC address**: Always port 0
- **Port 0**: The standard default for all TNCs

Most TNCs only support port 0. Multi-port support requires TNC hardware/firmware that supports multiple ports.

## Configuration File Format

### Serial Port Definitions

```ini
serial_portXXXX=/dev/ttyUSB0
serial_portXXXX_baud=9600
serial_portXXXX_flow_control=none|software|hardware|dtrdsr
serial_portXXXX_stop_bits=1|2
serial_portXXXX_parity=none|even|odd
serial_portXXXX_extended_kiss=false|true
```

### Cross-Connect Definitions

```ini
cross_connectXXXX=endpoint_a <-> endpoint_b
cross_connectXXXX_phil_flag=false|true
cross_connectXXXX_parse_kiss=false|true
```

**Endpoint Types**:
- `tcp:address:port` - TCP socket listener
- `serial:port_id:kiss_port` - Serial port with KISS port number (0-15)

## Quick Start Example

```ini
# Define a serial port
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600

# Connect it to TCP (KISS port 0 is default)
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
cross_connect0000_parse_kiss=true

log_level=5
```

## Common Examples

### Example 1: Kenwood TS-2000

```ini
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=4800
serial_port0000_stop_bits=2
serial_port0000_parity=none

cross_connect0000=serial:0000:0 <-> tcp:127.0.0.1:8001
cross_connect0000_phil_flag=true
cross_connect0000_parse_kiss=true
```

### Example 2: Multiple TNCs

```ini
# TNC 1 - VHF
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001

# TNC 2 - UHF
serial_port0001=/dev/ttyUSB1
serial_port0001_baud=9600
cross_connect0001=serial:0001:0 <-> tcp:0.0.0.0:8002

# TNC 3 - HF
serial_port0002=/dev/ttyUSB2
serial_port0002_baud=1200
cross_connect0002=serial:0002:0 <-> tcp:0.0.0.0:8003
```

### Example 3: Serial-to-Serial Bridge

```ini
# Bridge two TNCs together
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600

serial_port0001=/dev/ttyUSB1
serial_port0001_baud=9600

cross_connect0000=serial:0000:0 <-> serial:0001:0
cross_connect0000_parse_kiss=true
```

## Migration from v1.x

**Old format (v1.x)**:
```ini
serial_port=/dev/ttyUSB0
baud_rate=9600
tcp_address=0.0.0.0
tcp_port=8001
```

**New format (v2.0)**:
```ini
serial_port0000=/dev/ttyUSB0
serial_port0000_baud=9600
cross_connect0000=serial:0000:0 <-> tcp:0.0.0.0:8001
```

## Troubleshooting

### No packets received
- Verify KISS port 0 is used (default TNC address)
- Check serial port settings match TNC
- Enable `parse_kiss=true` to see frame flow

### Permission denied on serial port
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

### Packets corrupted
- Try enabling PhilFlag for TASCO modems or TS-2000
- Check baud rate and serial format (8N1 vs 8N2)

## Support

- **Repository**: https://github.com/ke4ahr/rax25kb
- **Issues**: https://github.com/ke4ahr/rax25kb/issues
