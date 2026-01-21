# TSTailnet

Represents a Tailscale network (tailnet) that contains devices and provides network configuration.

**Labels:** `:TSTailnet`

**Properties:**

- `id` - Tailnet name/identifier (primary key)
- `magicDnsEnabled` - Boolean indicating if MagicDNS is enabled
- `magicDnsSuffix` - DNS suffix used when MagicDNS is enabled

## Relationships

- **TSTailnet** → **CONTAINS** → **TSDevice** - Tailnets contain all connected devices