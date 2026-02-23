# PEERS_WITH

Represents a virtual network peering connection to a remote virtual network.

## Usage

- **NetworkPeering** → `PEERS_WITH` → **ArmResource** - A peering object connects to a remote virtual network

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all virtual network peering connections
MATCH (vnet:VirtualNetwork)-[:HAS_PEERING]->(p:NetworkPeering)-[:PEERS_WITH]->(remote:VirtualNetwork)
RETURN vnet.name, p.peeringState, remote.name
```

```cypher
// Find peerings that allow gateway transit
MATCH (vnet:VirtualNetwork)-[:HAS_PEERING]->(p:NetworkPeering)-[:PEERS_WITH]->(remote:VirtualNetwork)
WHERE p.allowGatewayTransit = true
RETURN vnet.name, remote.name
```
