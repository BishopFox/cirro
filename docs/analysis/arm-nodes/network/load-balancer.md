# LoadBalancer

Represents Azure Load Balancer resources.

**Labels:** `:ArmResource:LoadBalancer`

**Properties:**

- `id` - Load balancer resource ID (primary key)
- `resourceGuid` - Azure-generated unique resource GUID
- `provisioningState` - Current provisioning state

## Relationships

### Outgoing

- **LoadBalancer** -> `HAS_RULE` -> **LoadBalancerRule** - Load balancing rules attached to the load balancer
- **LoadBalancer** -> `HAS_LB_FRONTEND` -> **LoadBalancerFrontendIPConfiguration** - Frontend IP configurations
- **LoadBalancer** -> `HAS_BACKEND_POOL` -> **LoadBalancerBackendAddressPool** - Backend address pools

## Examples

```cypher
// Find load balancers with frontend and backend topology
MATCH (lb:LoadBalancer)
OPTIONAL MATCH (lb)-[:HAS_LB_FRONTEND]->(fe:LoadBalancerFrontendIPConfiguration)
OPTIONAL MATCH (lb)-[:HAS_BACKEND_POOL]->(be:LoadBalancerBackendAddressPool)
RETURN lb.name, count(DISTINCT fe) AS frontendCount, count(DISTINCT be) AS backendPoolCount
```
