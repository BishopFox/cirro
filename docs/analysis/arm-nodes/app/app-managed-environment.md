# AppManagedEnvironment

Represents Azure Container App managed environments.

**Labels:** `:ArmResource:AppManagedEnvironment`

**Properties:**

- `id` - Resource ID (primary key)
- `daprAIConnectionString` - Dapr Application Insights connection string
- `defaultDomain` - Default domain for the environment
- `eventStreamEndpoint` - Event stream endpoint
- `infrastructureResourceGroup` - Backing infrastructure resource group
- `mtls` - Mutual TLS enabled
- `encryption` - Peer traffic encryption enabled
- `staticIp` - Static IP assigned to the environment

## Relationships

### Outgoing
- **AppManagedEnvironment** → `DEPLOYS` → **ContainerApp** - Container Apps deployed into this environment

## Examples

```cypher
// Find environments and their apps
MATCH (env:AppManagedEnvironment)-[:DEPLOYS]->(app:ContainerApp)
RETURN env.defaultDomain, collect(app.fqdn) AS apps
```
