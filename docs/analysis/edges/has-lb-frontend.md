# HAS_LB_FRONTEND

Represents the relationship between a load balancer and its frontend IP configurations.

## Usage

- **LoadBalancer** -> `HAS_LB_FRONTEND` -> **LoadBalancerFrontendIPConfiguration** - Frontend configuration attached to a load balancer

## Properties

No additional properties on the relationship.

## Examples

```cypher
MATCH (lb:LoadBalancer)-[:HAS_LB_FRONTEND]->(fe:LoadBalancerFrontendIPConfiguration)
RETURN lb.name, fe.name, fe.privateIPAllocationMethod
```
