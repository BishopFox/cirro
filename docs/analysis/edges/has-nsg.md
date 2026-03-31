# HAS_NSG

Represents network security group associations with network interfaces.

**Direction:** `(networkInterface)-[:HAS_NSG]->(nsg)`

**Description:** Indicates that a network interface is associated with a network security group for traffic filtering and security rules.

**Common Patterns:**

- Network interfaces have associated network security groups
- NSGs provide network-level security through rules
- Multiple network interfaces can share the same NSG

**Properties:** None

## Query Examples

```cypher
// Find all network interfaces with their NSGs
MATCH path = (nic:NetworkInterface)-[:HAS_NSG]->(nsg:NSG)
RETURN path

// Find virtual machines and their network security through NICs
MATCH path = (vm:VirtualMachine)-[:HAS_NIC]->(nic:NetworkInterface)-[:HAS_NSG]->(nsg:NSG)
RETURN path

// Find network security rules applied to network interfaces
MATCH path = (nic:NetworkInterface)-[:HAS_NSG]->(nsg:NSG)-[:HAS_RULE]->(rule:NSGRule)
RETURN path
```
