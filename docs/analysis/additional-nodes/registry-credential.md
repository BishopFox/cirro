# RegistryCredential

Represents container image registry credentials associated with container groups.

**Labels:** `:RegistryCredential`

**Properties:**

- `username` - Registry username (primary key component)
- `server` - Registry server name (primary key component)
- `isDelegatedIdentity` - Whether delegated identity is used

## Relationships

### Outgoing

- **RegistryCredential** → `AUTHENTICATES` → **ContainerGroup** - Credential used to authenticate pulls for a container group

## Examples

```cypher
// Find registry credentials used by container groups
MATCH (cred:RegistryCredential)-[:AUTHENTICATES]->(cg:ContainerGroup)
RETURN cred.server, cred.username, cred.isDelegatedIdentity, cg.name
```
