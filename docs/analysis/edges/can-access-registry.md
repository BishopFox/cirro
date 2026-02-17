# CAN_ACCESS_REGISTRY

Represents a Container App's ability to pull from a Container Registry with specific credentials.

## Usage

- **ContainerApp** → `CAN_ACCESS_REGISTRY` → **ContainerRegistry**

### Properties
- `identity` - Managed identity used for the pull (if any)
- `username` - Registry username (if provided)
- `passwordSecretRef` - Secret reference for the password (if provided)

## Examples

```cypher
MATCH (app:ContainerApp)-[r:CAN_ACCESS_REGISTRY]->(acr:ContainerRegistry)
RETURN app.fqdn, acr.loginServer, r.identity, r.username
```
