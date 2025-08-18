# HAS_GATEWAY

Represents the relationship between network peering connections and their gateways.

## Usage

This relationship connects network peering to their gateway configurations:

- **NetworkPeering** → `HAS_GATEWAY` → **NetworkGateway** - Network peering to their gateways

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all peering connections with gateways
MATCH (peering:NetworkPeering)-[:HAS_GATEWAY]->(gateway:NetworkGateway)
RETURN peering.name, gateway.gatewayType
```
