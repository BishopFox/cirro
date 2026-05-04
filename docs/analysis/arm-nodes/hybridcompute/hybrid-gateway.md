# HybridGateway

Represents an Azure Arc gateway resource used to connect hybrid machines to Azure through a managed endpoint.

**Labels:** `:ArmResource:HybridGateway`

**Properties:**

- `id` - Gateway resource ID (primary key)
- `allowedFeatures` - Features allowed through the gateway
- `gatewayEndpoint` - Gateway endpoint URL
- `gatewayId` - Unique gateway identifier
- `gatewayType` - Type of gateway
- `lastUpdateTime` - Last update timestamp
- `provisioningState` - Provisioning state

## Examples

```cypher
// Find all hybrid gateways
MATCH (gw:HybridGateway)
RETURN gw.id, gw.gatewayType, gw.gatewayEndpoint, gw.provisioningState
```
