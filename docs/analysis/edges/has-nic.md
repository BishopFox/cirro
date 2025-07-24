# HAS_NIC

Represents network interface attachment relationships between virtual machines and network interfaces.

**Direction:** `(virtualMachine)-[:HAS_NIC]->(networkInterface)`

**Description:** Indicates that a virtual machine has an attached network interface for network connectivity.

**Common Patterns:**

- Virtual machines have one or more network interfaces
- Network interfaces provide network connectivity to VMs
- Multiple NICs can be attached to a single VM for multi-homing scenarios

**Properties:** None

## Query Examples

```cypher
// Find all network interfaces attached to virtual machines
MATCH path = (vm:VirtualMachine)-[:HAS_NIC]->(nic:NetworkInterface)
RETURN path

// Find virtual machines with multiple network interfaces
MATCH (vm:VirtualMachine)-[:HAS_NIC]->(nic:NetworkInterface)
WITH vm, COUNT(nic) as nicCount
WHERE nicCount > 1
RETURN vm, nicCount
ORDER BY nicCount DESC

// Find network topology from VM to subnet
MATCH path = (vm:VirtualMachine)-[:HAS_NIC]->(nic:NetworkInterface)-[:HAS_IPCONFIG]->(ipconfig:IpConfiguration)-[:LOCATED_IN]->(subnet:Subnet)
RETURN path

// Find VMs and their network security through NICs
MATCH path = (vm:VirtualMachine)-[:HAS_NIC]->(nic:NetworkInterface)-[:HAS_NSG]->(nsg:NSG)
RETURN path

// Find VM network configuration details
MATCH (vm:VirtualMachine)-[:HAS_NIC]->(nic:NetworkInterface)
RETURN vm.name, nic.name, nic.location, nic.enableIPForwarding, nic.enableAcceleratedNetworking
```
