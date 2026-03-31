# NetworkPeering

Represents virtual network peering connections (not an ARM resource itself).

**Labels:** `:NetworkPeering`

**Properties:**

- `id` - Peering ID (primary key)
- `name` - Peering name
- `allowVirtualNetworkAccess` - Whether virtual network access is allowed
- `allowForwardedTraffic` - Whether forwarded traffic is allowed
- `allowGatewayTransit` - Whether gateway transit is allowed
- `useRemoteGateways` - Whether to use remote gateways
- `peeringState` - Current state of the peering (Connected, Disconnected, etc.)

## Relationships

### Incoming

- **VirtualNetwork** → `HAS_PEERING` → **NetworkPeering** - Source virtual network

### Outgoing

- **NetworkPeering** → `PEERS_WITH` → **ArmResource** - Target virtual network

## Examples

```cypher
// Find all connected peering relationships
MATCH (vnet:VirtualNetwork)-[:HAS_PEERING]->(peering:NetworkPeering)
WHERE peering.peeringState = "Connected"
RETURN vnet.name, peering.name, peering.allowForwardedTraffic
```

```cypher
// Find bidirectional peering relationships
MATCH (vnet1:VirtualNetwork)-[:HAS_PEERING]->(p1:NetworkPeering)-[:PEERS_WITH]->(vnet2:VirtualNetwork)
MATCH (vnet2)-[:HAS_PEERING]->(p2:NetworkPeering)-[:PEERS_WITH]->(vnet1)
RETURN vnet1.name, vnet2.name, p1.peeringState, p2.peeringState
```
