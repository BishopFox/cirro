# IPConfiguration

Represents IP configurations for network interfaces (not an ARM resource itself).

**Labels:** `:IPConfiguration`

**Properties:**

- `id` - IP configuration ID (primary key)
- `name` - Configuration name
- `type` - Configuration type
- `privateIPAddress` - Private IP address
- `privateIPAddressVersion` - IP address version (IPv4/IPv6)
- `privateIPAllocationMethod` - Private IP allocation method (Static/Dynamic)

## Relationships

### Incoming

- **NetworkInterface** → `HAS_CONFIG` → **IPConfiguration** - Parent network interface
- **Subnet** → `CONTAINS` → **IPConfiguration** - Subnet containing this IP configuration

### Outgoing

- **IPConfiguration** → `HAS_IP` → **PublicIPAddress** - Associated public IP address (if any)

## Examples

```cypher
// Find all static IP configurations
MATCH (ip:IPConfiguration)
WHERE ip.privateIPAllocationMethod = "Static"
RETURN ip.name, ip.privateIPAddress, ip.privateIPAddressVersion
```

```cypher
// Find IP configurations with public IPs
MATCH (ip:IPConfiguration)-[:HAS_IP]->(pub:PublicIPAddress)
RETURN ip.privateIPAddress, pub.ipAddress
```
