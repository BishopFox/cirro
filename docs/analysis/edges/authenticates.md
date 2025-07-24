# AUTHENTICATES

Represents authentication credential relationships.

**Direction:** `(clientSecret/certificate)-[:AUTHENTICATES]->(application/servicePrincipal)`

**Description:** Indicates that a client secret or certificate can be used to authenticate as an application or service principal.

**Properties:** None

## Query Examples

```cypher
// Find all authentication methods for an application
MATCH path=(cred)-[:AUTHENTICATES]->(app:GraphApplication)
RETURN path

// Find applications using certificate authentication
MATCH path=(cert:Certificate)-[:AUTHENTICATES]->(app:GraphApplication)
RETURN path

// Find applications with multiple authentication methods
MATCH (cred)-[:AUTHENTICATES]->(app:GraphApplication)
WITH app, COUNT(cred) as credCount
WHERE credCount > 1
RETURN app, credCount
```
