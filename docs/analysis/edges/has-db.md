# HAS_DB

Represents database ownership relationships in SQL Server environments.

**Direction:** `(sqlServer)-[:HAS_DB]->(sqlDatabase)`

**Description:** Indicates that a SQL Server contains or manages a specific SQL Database.

**Common Patterns:**

- SQL Servers contain multiple databases
- Each database is managed by exactly one SQL Server
- Server-level configurations and security apply to all contained databases

**Properties:** None

## Query Examples

```cypher
// Find all databases on a SQL Server
MATCH path = (server:SqlServer)-[:HAS_DB]->(db:SqlDatabase)
RETURN path

// Find SQL Servers with many databases
MATCH (server:SqlServer)-[:HAS_DB]->(db:SqlDatabase)
WITH server, COUNT(db) as dbCount
WHERE dbCount > 5
RETURN server, dbCount
ORDER BY dbCount DESC

// Find databases and their server properties
MATCH (server:SqlServer)-[:HAS_DB]->(db:SqlDatabase)
RETURN server.name, server.location, db.name, db.collation

// Find orphaned databases (databases without servers)
MATCH (db:SqlDatabase)
WHERE NOT (db)<-[:HAS_DB]-()
RETURN db
```
