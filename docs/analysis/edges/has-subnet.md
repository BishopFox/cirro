# HAS_SUBNET

Represents the relationship between virtual networks and their subnets.

## Usage

This relationship connects virtual networks to their contained subnets:

- **VirtualNetwork** → `HAS_SUBNET` → **Subnet** - Virtual networks to their subnets

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all virtual networks and their subnets
MATCH (vnet:VirtualNetwork)-[:HAS_SUBNET]->(subnet:Subnet)
RETURN vnet.name, subnet.name, subnet.addressPrefix
```
