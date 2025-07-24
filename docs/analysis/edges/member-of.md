# MEMBER_OF

Represents membership relationships between Graph objects.

**Direction:** `(object)-[:MEMBER_OF]->(group)`

**Description:** Indicates that a user, device, or other object is a member of a group.

**Common Patterns:**

- Users are members of groups
- Devices are members of groups
- Objects are members of administrative units

**Properties:** None

## Query Examples

```cypher
// Find all users in a specific group
MATCH path=(u:GraphUser)-[:MEMBER_OF]->(g:GraphGroup {displayName: 'Administrators'})
RETURN path

// Find all group memberships for a user
MATCH path=(u:GraphUser {userPrincipalName: 'john.doe@company.com'})-[:MEMBER_OF]->(g:GraphGroup)
RETURN path

// Find nested group memberships
MATCH path=(u:GraphUser)-[:MEMBER_OF*1..3]->(g:GraphGroup)
RETURN path
```
