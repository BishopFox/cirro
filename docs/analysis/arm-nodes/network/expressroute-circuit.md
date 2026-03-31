# ExpressRouteCircuit

Represents Azure ExpressRoute Circuit resources.

**Labels:** `:ArmResource:ExpressRouteCircuit`

**Properties:**

- `id` - Resource ID (primary key)
- `allowClassicOperations` - Whether classic operations are allowed
- `allowGlobalReach` - Whether Global Reach is allowed
- `authorizationKey` - Authorization key
- `circuitProvisioningState` - Circuit provisioning state
- `serviceProviderProvisioningState` - Service provider provisioning state
- `bandwidthInMbps` - Bandwidth in Mbps
- `peeringLocation` - Peering location
- `serviceProviderName` - Service provider name

## Relationships

### Outgoing

- **ExpressRouteCircuit** → `HAS_PEERING` → **ExpressRouteCircuitPeering** - Circuit peering configurations

## Examples

```cypher
// Find ExpressRoute circuits and their peerings
MATCH (erc:ExpressRouteCircuit)-[:HAS_PEERING]->(p:ExpressRouteCircuitPeering)
RETURN erc.name, p.peeringType, p.state, erc.bandwidthInMbps
```
