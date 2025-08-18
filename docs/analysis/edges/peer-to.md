# PEER_TO

Represents the peering relationship between virtual networks.

## Usage

This relationship connects network peering configurations to their target virtual networks:

- **NetworkPeering** → `PEER_TO` → **VirtualNetwork** - Network peering to the target virtual network

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all virtual network peering relationships
MATCH (vnet1:VirtualNetwork)-[:HAS_PEERING]->(peering:NetworkPeering)-[:PEER_TO]->(vnet2:VirtualNetwork)
RETURN vnet1.name as SourceVNet, vnet2.name as TargetVNet, peering.peeringState
```
