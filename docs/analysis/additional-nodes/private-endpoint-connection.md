# PrivateEndpointConnection

Represents an embedded private endpoint connection object found on ARM resources.

**Labels:** `:PrivateEndpointConnection`

**Properties:**

- `id` - Private endpoint connection ID (primary key)
- `name` - Connection name
- `type` - Resource type
- `groupIds` - Target group IDs used by the connection

## Relationships

### Incoming

- **ArmResource** → `HAS_PRIVATE_ENDPOINT` → **PrivateEndpointConnection** - Parent resource references this connection object

## Examples

```cypher
// Find resources with private endpoint connections
MATCH (r:ArmResource)-[:HAS_PRIVATE_ENDPOINT]->(p:PrivateEndpointConnection)
RETURN r.name, r.type, p.name, p.groupIds
```
