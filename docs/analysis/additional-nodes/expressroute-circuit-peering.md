# ExpressRouteCircuitPeering

Represents an ExpressRoute peering object attached to an ExpressRoute circuit.

**Labels:** `:ExpressRouteCircuitPeering`

**Properties:**

- `id` - Peering resource ID (primary key)
- `name` - Peering name
- `azureASN` - Azure ASN value
- `peerASN` - Customer/provider ASN value
- `primaryPeerAddressPrefix` - Primary peering CIDR
- `secondaryPeerAddressPrefix` - Secondary peering CIDR
- `primaryAzurePort` - Primary Azure port
- `secondaryAzurePort` - Secondary Azure port
- `vlanId` - VLAN ID used for peering
- `peeringType` - Peering type
- `state` - Peering state

## Relationships

### Incoming

- **ExpressRouteCircuit** → `HAS_PEERING` → **ExpressRouteCircuitPeering** - Circuit-to-peering linkage

## Examples

```cypher
// List ExpressRoute circuits and their peerings
MATCH (c:ExpressRouteCircuit)-[:HAS_PEERING]->(p:ExpressRouteCircuitPeering)
RETURN c.name, p.name, p.peeringType, p.state
```
