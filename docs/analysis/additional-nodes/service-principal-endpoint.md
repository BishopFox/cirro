# ServicePrincipalEndpoint

Represents service principal endpoints (not an ARM resource itself).

**Labels:** `:ServicePrincipalEndpoint`

**Properties:**

- `id` - Endpoint ID (primary key)
- `type` - Endpoint type
- `uri` - Endpoint URI
- `customKeyIdentifier` - Custom key identifier
- `displayName` - Endpoint display name
- `usage` - Endpoint usage description

## Relationships

### Incoming

- **ServicePrincipal** → `HAS_ENDPOINT` → ServicePrincipalEndpoint - Parent service principal

## Examples

```cypher
// Find all service principal endpoints
MATCH (sp:ServicePrincipal)-[:HAS_ENDPOINT]->(endpoint:ServicePrincipalEndpoint)
RETURN sp.displayName, endpoint.type, endpoint.uri, endpoint.displayName
```

```cypher
// Find service principals with specific endpoint types
MATCH (sp:ServicePrincipal)-[:HAS_ENDPOINT]->(endpoint:ServicePrincipalEndpoint)
WHERE endpoint.type = "oauth2"
RETURN sp.displayName, endpoint.uri
```