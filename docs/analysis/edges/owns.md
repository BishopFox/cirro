# OWNS

Represents ownership relationships in Entra ID.

**Direction:** `(user/servicePrincipal)-[:OWNS]->(application/group/servicePrincipal)`

**Description:** Indicates that a user or service principal owns an application, group, or other service principal.

**Common Patterns:**
- Users own applications they created
- Service principals own other service principals
- Users own groups they manage

**Properties:** None

## Query Examples

```cypher
// Find all applications owned by a user
MATCH path=(u:GraphUser)-[:OWNS]->(app:GraphApplication)
RETURN path

// Find all resources owned by service principals
MATCH path=(sp:GraphServicePrincipal)-[:OWNS]->(resource)
RETURN path
```
