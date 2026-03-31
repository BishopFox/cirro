# HAS_USER_GRANT

Represents a client application's user-delegated permission grant to a resource.

**Direction:** `(servicePrincipal)-[:HAS_USER_GRANT]->(servicePrincipal)`

**Description:** Indicates that a client service principal has a user-delegated permission grant to access a resource service principal. Created alongside a `DELEGATED_TO_CLIENT` relationship when a specific user consents to delegated permissions — the user delegates to the client, and the client has a user grant to the resource.

**Common Patterns:**
- Always paired with a `DELEGATED_TO_CLIENT` from the consenting user to the client
- Represents the client-to-resource leg of a user-consented delegated permission

**Properties:** None

## Query Examples

```cypher
// Find all user-delegated grants from clients to resources
MATCH path=(client:GraphServicePrincipal)-[:HAS_USER_GRANT]->(resource:GraphServicePrincipal)
RETURN path

// Trace the full delegation chain: user -> client -> resource
MATCH (u:GraphUser)-[:DELEGATED_TO_CLIENT]->(client:GraphServicePrincipal)-[:HAS_USER_GRANT]->(resource:GraphServicePrincipal)
RETURN u.displayName, client.displayName, resource.displayName

// Find resources with the most user-delegated grants
MATCH (client:GraphServicePrincipal)-[:HAS_USER_GRANT]->(resource:GraphServicePrincipal)
RETURN resource.displayName, COUNT(client) AS grantCount
ORDER BY grantCount DESC
```
