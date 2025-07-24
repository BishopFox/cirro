# REPRESENTED_BY

Represents the relationship between applications and service principals.

**Direction:** `(application)-[:REPRESENTED_BY]->(servicePrincipal)`

**Description:** Indicates that an application is represented by a service principal in the directory.

**Properties:** None

## Query Examples

```cypher
// Find service principal for an application
MATCH path=(app:GraphApplication)-[:REPRESENTED_BY]->(sp:GraphServicePrincipal)
RETURN path

// Find applications without service principals
MATCH (app:GraphApplication)
WHERE NOT (app)-[:REPRESENTED_BY]->()
RETURN app

// Find service principals without applications
MATCH (sp:GraphServicePrincipal)
WHERE NOT (sp) <-[:REPRESENTED_BY]- ()
RETURN sp
```
