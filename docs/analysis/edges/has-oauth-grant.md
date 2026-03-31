# HAS_OAUTH_GRANT

Represents tenant-level (admin consent) OAuth2 permission grants between service principals.

**Direction:** `(servicePrincipal)-[:HAS_OAUTH_GRANT]->(servicePrincipal)`

**Description:** Indicates that a client service principal has been granted OAuth2 permissions to a resource service principal at the tenant level. This typically represents admin consent where no specific user principal is involved — the grant applies across the entire tenant.

**Common Patterns:**
- Admin-consented API permissions between applications
- Tenant-wide delegated permission grants (no specific user)

**Properties:**

- `id` - OAuth2 permission grant ID
- `scopes` - List of delegated permission scopes granted
- `consentType` - Type of consent (e.g., `AllPrincipals`)
- `principalId` - Always null for tenant-level grants
- `startTime` - When the grant was created
- `expiryTime` - When the grant expires

## Query Examples

```cypher
// Find all tenant-level OAuth2 grants
MATCH path=(client:GraphServicePrincipal)-[:HAS_OAUTH_GRANT]->(resource:GraphServicePrincipal)
RETURN path

// Find applications with broad admin-consented scopes
MATCH (client:GraphServicePrincipal)-[r:HAS_OAUTH_GRANT]->(resource:GraphServicePrincipal)
WHERE size(r.scopes) > 5
RETURN client.displayName, resource.displayName, r.scopes

// Find all admin-consented grants to Microsoft Graph
MATCH (client:GraphServicePrincipal)-[r:HAS_OAUTH_GRANT]->(resource:GraphServicePrincipal)
WHERE resource.displayName = "Microsoft Graph"
RETURN client.displayName, r.scopes
ORDER BY size(r.scopes) DESC
```
