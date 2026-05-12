# VirtualHub

Represents Azure Virtual Hub resources.

**Labels:** `:ArmResource:VirtualHub`

**Properties:**

- `id` - Virtual hub resource ID (primary key)
- `allowBranchToBranchTraffic` - Whether branch-to-branch traffic is allowed
- `hubRoutingPreference` - Hub routing preference mode
- `provisioningState` - Current provisioning state
- `routingState` - Current routing state
- `virtualRouterAsn` - ASN used by the virtual router
- `virtualRouterIps` - Virtual router IP addresses

## Relationships

No direct relationships are created in ingestion for this node type.

## Examples

```cypher
// Find virtual hubs and routing posture
MATCH (vh:VirtualHub)
RETURN vh.name, vh.hubRoutingPreference, vh.routingState, vh.virtualRouterAsn
```
