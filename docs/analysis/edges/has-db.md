# HAS_DB

Represents database ownership relationships in SQL Server environments.

**Direction:** `(sqlServer)-[:HAS_DB]->(sqlDatabase)`

**Description:** Indicates that a SQL Server contains or manages a specific SQL Database. This applies to both traditional SQL Servers and Azure Arc-enabled SQL Server instances.

**Common Patterns:**

- SQL Servers contain multiple databases
- Azure Arc SQL Servers contain Arc-enabled databases
- Each database is managed by exactly one SQL Server
- Server-level configurations and security apply to all contained databases

**Properties:** None

## Usage

This relationship connects SQL Server instances to their databases:

- **SqlServer** → `HAS_DB` → **SqlDatabase** - Traditional SQL Server databases
- **ArcSqlServer** → `HAS_DB` → **ArcSqlDatabase** - Azure Arc-enabled SQL Server databases

## Query Examples

```cypher
// Find all databases on a SQL Server
MATCH path = (server:SqlServer)-[:HAS_DB]->(db:SqlDatabase)
RETURN path

// Find all databases on Azure Arc SQL Servers
MATCH path = (server:ArcSqlServer)-[:HAS_DB]->(db:ArcSqlDB)
RETURN path

// Find SQL Servers with many databases
MATCH (server:SqlServer)-[:HAS_DB]->(db:SqlDatabase)
WITH server, COUNT(db) as dbCount
WHERE dbCount > 5
RETURN server, dbCount
ORDER BY dbCount DESC

// Find Arc SQL databases and their properties
MATCH (server:ArcSqlServer)-[:HAS_DB]->(db:ArcSqlDB)
RETURN server.instanceName, db.name, db.dataFileSizeMB, db.state

// Find databases and their server properties
MATCH (server:SqlServer)-[:HAS_DB]->(db:SqlDatabase)
RETURN server.name, server.location, db.name, db.collation

// Find orphaned databases (databases without servers)
MATCH (db)
WHERE (db:SqlDatabase OR db:ArcSqlDB) AND NOT (db)<-[:HAS_DB]-()
RETURN db
```
