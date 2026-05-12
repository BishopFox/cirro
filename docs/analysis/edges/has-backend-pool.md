# HAS_BACKEND_POOL

Represents the relationship between a load balancer and its backend address pools.

## Usage

- **LoadBalancer** -> `HAS_BACKEND_POOL` -> **LoadBalancerBackendAddressPool** - Backend pool attached to a load balancer

## Properties

No additional properties on the relationship.

## Examples

```cypher
MATCH (lb:LoadBalancer)-[:HAS_BACKEND_POOL]->(pool:LoadBalancerBackendAddressPool)
RETURN lb.name, pool.name, pool.provisioningState
```
