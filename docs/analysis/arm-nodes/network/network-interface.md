# NetworkInterface

Represents network interfaces attached to VMs.

**Labels:** `:ArmResource:NetworkInterface`

**Properties:**

- `id` - NIC resource ID (primary key)
- `macAddress` - MAC address
- `allowPort25Out` - Whether port 25 outbound is allowed
- `appliedDnsServers` - Applied DNS servers
- `dnsServers` - Configured DNS servers

## Relationships

### Incoming

- **VirtualMachine** → `HAS_NIC` → **NetworkInterface** - VM using this network interface

### Outgoing

- **NetworkInterface** → `HAS_NSG` → **NetworkSecurityGroup** - Associated network security group
- **NetworkInterface** → `HAS_CONFIG` → **IPConfiguration** - IP configurations

## Examples

```cypher
// Find network interfaces and their VMs
MATCH (vm:VirtualMachine)-[:HAS_NIC]->(nic:NetworkInterface)
RETURN vm.computerName, nic.macAddress, nic.allowPort25Out
```

```cypher
// Find network interfaces with their IP configurations
MATCH (nic:NetworkInterface)-[:HAS_CONFIG]->(ip:IPConfiguration)
RETURN nic.name, ip.privateIPAddress, ip.privateIPAllocationMethod
```
