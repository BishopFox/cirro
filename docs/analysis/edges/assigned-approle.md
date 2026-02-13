# ASSIGNED_APPROLE

Represents app role assignments from a Graph object to a target service principal.

**Direction:** `(graphObject)-[:ASSIGNED_APPROLE]->(servicePrincipal)`

**Description:** Created from `appRoleAssignedTo` data. The relationship links principals to the service principals where app roles are assigned.

**Properties:**

- `appRoleId` - App role ID assigned on the target service principal
- `displayName` - Resolved app role display name (post-processing)
- `description` - Resolved app role description (post-processing)
- `isEnabled` - Whether the role is enabled
- `isPreAuthorizationRequired` - Pre-auth requirement flag
- `isPrivate` - Private role flag
- `origin` - Role origin
- `value` - Role value

## Query Examples

```cypher
// Find users with app role assignments
MATCH (u:GraphUser)-[r:ASSIGNED_APPROLE]->(sp:GraphServicePrincipal)
RETURN u.displayName, sp.displayName, r.appRoleId, r.value
```
