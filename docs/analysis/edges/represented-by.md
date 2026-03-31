# REPRESENTED_BY

Represents the relationship between applications and service principals.

**Direction:** `(application)-[:REPRESENTED_BY]->(servicePrincipal)`

**Description:** Indicates that an identity is represented by a service principal or other identity object in the directory.

**Properties:** None

## Query Examples

```cypher
// Find service principal for an application
MATCH path=(app:GraphApplication)-[:REPRESENTED_BY]->(sp:GraphServicePrincipal)
RETURN path

// Find the service principal for a user-assigned identity
MATCH path=(identity:UserAssignedIdentity)-[:REPRESENTED_BY]->(sp:GraphServicePrincipal)
RETURN path

// Find service principals without applications
MATCH (sp:GraphServicePrincipal)
WHERE NOT (sp) <-[:REPRESENTED_BY]- ()
RETURN sp
```
