# VirtualNetwork

Represents Azure virtual networks.

**Labels:** `:ArmResource:VirtualNetwork`

**Properties:**

- `id` - VNet resource ID (primary key)
- `resourceGuid` - Resource GUID
- `provisioningState` - Provisioning state
- `enableDdosProtection` - Whether DDoS protection is enabled
- `enableVmProtection` - Whether VM protection is enabled
- `addressSpace` - Array of address prefixes for the VNet

## Relationships

### Outgoing

- **VirtualNetwork** → `CONTAINS` → **Subnet** - Subnets within the virtual network
- **VirtualNetwork** → `HAS_PEERING` → **NetworkPeering** - Network peering connections

## Examples

```cypher
// Find all virtual networks with DDoS protection
MATCH (vnet:VirtualNetwork)
WHERE vnet.enableDdosProtection = true
RETURN vnet.name, vnet.addressSpace
```

```cypher
// Find virtual networks and their subnets
MATCH (vnet:VirtualNetwork)-[:CONTAINS]->(subnet:Subnet)
RETURN vnet.name, collect(subnet.name) AS subnets
```

```cypher
// Find virtual network peering relationships
MATCH (vnet:VirtualNetwork)-[:HAS_PEERING]->(peering:NetworkPeering)-[:PEERS_WITH]->(remote:ArmResource)
RETURN vnet.name, remote.name, peering.peeringState
```
