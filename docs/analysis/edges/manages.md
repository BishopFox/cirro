# MANAGES

Represents management relationships between resources.

**Direction:** `(manager)-[:MANAGES]->(managed)`

**Description:** Indicates that one resource manages or controls another resource.

**Common Patterns:**
- Automation accounts manage runbooks
- SQL servers manage databases
- Entra ID managers manage enabled member users

**Properties:** None

## Query Examples

```cypher
// Find all resources managed by automation accounts
MATCH path=(aa:AutomationAccount)-[:MANAGES]->(resource)
RETURN path

// Find databases managed by SQL servers
MATCH path=(server:SqlServer)-[:MANAGES]->(db:SqlDatabase)
RETURN path

// Find management relationships
MATCH path=(manager)-[:MANAGES]->(managed)
RETURN path

// Find Entra ID users and their managers
MATCH path=(manager:GraphUser)-[:MANAGES]->(user:GraphUser)
RETURN path
```
