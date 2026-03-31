# Eligible Role Assignments

Represents eligible role assignments from Microsoft Graph PIM (Privileged Identity Management).

**Labels:** `:GraphObject`, `:GraphRole`

This configuration creates `IS_ROLE_ELIGIBLE` relationships between principals and their eligible Graph roles.

**Relationship Properties (on IS_ROLE_ELIGIBLE):**

- `createdDateTime` - Creation date/time
- `directoryScopeId` - Directory scope ID
- `memberType` - Member type
- `modifiedDateTime` - Modified date/time
- `expirationType` - Expiration type
- `startDateTime` - Start date/time
- `endDateTime` - End date/time
- `recurrence` - Recurrence schedule

## Relationships

- **GraphObject** → `IS_ROLE_ELIGIBLE` → **GraphRole** - Principal is eligible for the role

## Examples

```cypher
// Find users eligible for directory roles
MATCH (u:GraphUser)-[r:IS_ROLE_ELIGIBLE]->(role:GraphRole)
RETURN u.displayName, role.displayName, r.expirationType, r.endDateTime
```
