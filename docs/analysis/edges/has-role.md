# HAS_ROLE

Represents role assignments in Entra ID.

**Direction:** `(user)-[:HAS_ROLE]->(role)`

**Description:** Indicates that a user or service principal has been assigned a specific role or role within an administrative unit.

**Common Patterns:**

- Users have directory roles (Global Administrator, User Administrator, etc.)
- Service principals have role assignments
- Objects have scoped roles within administrative units

**Properties:**

- `roleId` - ID of the assigned role
- `roleName` - Display name of the role (when available)

## Query Examples

```cypher
// Find all Global Administrators
MATCH path=(u:GraphUser)-[r:HAS_ROLE]->(role {displayName: "Global Administrator"})
RETURN path

// Find all role assignments for a user
MATCH path=(u:GraphUser {userPrincipalName: 'admin@company.com'})-[r:HAS_ROLE]->(role)
RETURN path

// Find users with multiple roles
MATCH (u:GraphUser)-[r:HAS_ROLE]->(role)
WITH u, COUNT(r) as roleCount
WHERE roleCount > 1
RETURN u, roleCount
```
