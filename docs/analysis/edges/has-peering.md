# HAS_PEERING

Represents the relationship between virtual networks and their peering connections.

## Usage

This relationship connects virtual networks to their peering configurations:

- **VirtualNetwork** → `HAS_PEERING` → **NetworkPeering** - Virtual networks to their peering connections

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all virtual networks and their peering connections
MATCH (vnet:VirtualNetwork)-[:HAS_PEERING]->(peering:NetworkPeering)
RETURN vnet.name, peering.name, peering.peeringState
```
