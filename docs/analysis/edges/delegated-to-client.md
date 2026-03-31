# DELEGATED_TO_CLIENT

Represents user-specific delegated OAuth2 permission grants.

**Direction:** `(user)-[:DELEGATED_TO_CLIENT]->(servicePrincipal)`

**Description:** Indicates that a user has granted delegated OAuth2 permissions to a client application (service principal). This relationship is created when `principalId` is present on the OAuth2 permission grant, meaning a specific user consented to the application acting on their behalf.

**Common Patterns:**
- Users consent to applications requesting delegated permissions
- Created alongside a corresponding `HAS_USER_GRANT` relationship from the client to the resource

**Properties:**

- `id` - OAuth2 permission grant ID
- `scopes` - List of delegated permission scopes granted
- `consentType` - Type of consent (e.g., `Principal`)
- `principalId` - The user who granted consent
- `startTime` - When the grant was created
- `expiryTime` - When the grant expires

## Query Examples

```cypher
// Find all applications a user has delegated permissions to
MATCH path=(u:GraphUser)-[:DELEGATED_TO_CLIENT]->(sp:GraphServicePrincipal)
RETURN path

// Find users who have delegated Mail.Read scope
MATCH (u:GraphUser)-[r:DELEGATED_TO_CLIENT]->(sp:GraphServicePrincipal)
WHERE "Mail.Read" IN r.scopes
RETURN u.displayName, sp.displayName, r.scopes

// Find users with the most delegated grants
MATCH (u:GraphUser)-[r:DELEGATED_TO_CLIENT]->(sp:GraphServicePrincipal)
RETURN u.displayName, COUNT(sp) AS grantCount
ORDER BY grantCount DESC
```
