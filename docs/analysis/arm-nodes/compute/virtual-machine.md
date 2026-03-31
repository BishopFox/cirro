# VirtualMachine

Represents Azure virtual machines.

**Labels:** `:ArmResource:VirtualMachine`

**Properties:**

- `id` - VM resource ID (primary key)
- `vmSize` - VM size/SKU
- `adminUsername` - Administrator username
- `allowExtensionOperations` - Whether extensions are allowed
- `computerName` - Computer name
- `vmId` - Virtual machine ID
- `os` - Operating system type
- `image` - VM image offer
- `imageVersion` - VM image version

## Relationships

### Outgoing

- **VirtualMachine** → `HAS_NIC` → **NetworkInterface** - Network interfaces attached to the VM
- **VirtualMachine** → `HAS_EXTENSION` → **VMExtension** - Extensions installed on the VM

## Examples

```cypher
// Find all VMs with their sizes
MATCH (vm:VirtualMachine)
RETURN vm.computerName, vm.vmSize, vm.os
```

```cypher
// Find VMs and their network interfaces
MATCH (vm:VirtualMachine)-[:HAS_NIC]->(nic:NetworkInterface)
RETURN vm.computerName, collect(nic.name) AS networkInterfaces
```

```cypher
// Find VMs with extensions
MATCH (vm:VirtualMachine)-[:HAS_EXTENSION]->(ext:VMExtension)
RETURN vm.computerName, ext.name, ext.type
```