# HAS_IP

Represents relationships between network components and their IP addresses.

## Usage

This relationship connects network resources to their assigned IP addresses:

- **IPConfig** → `HAS_IP` → **PublicIPAddress** - IP configurations to their public IP addresses
- **BastionIPConfig** → `HAS_IP` → **PublicIPAddress** - Bastion IP configurations to public IPs
- **HybridMachine** → `HAS_IP` → **HybridIPAddress** - Hybrid machines to their IP addresses

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all resources with public IP addresses
MATCH (resource)-[:HAS_CONFIG]->(config)-[:HAS_IP]->(ip:PublicIPAddress)
RETURN resource.name, ip.ipAddress
```
