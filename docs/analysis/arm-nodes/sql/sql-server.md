# SqlServer

Represents Azure SQL servers.

**Labels:** `:ArmResource:SqlServer`

**Properties:**

- `id` - SQL server resource ID (primary key)
- `administratorLogin` - Administrator login name
- `fullyQualifiedDomainName` - Fully qualified domain name
- `publicNetworkAccess` - Public network access setting
- `restrictOutboundNetworkAccess` - Whether outbound network access is restricted
- `state` - Server state
- `version` - SQL Server version

## Relationships

### Outgoing

- **SqlServer** → `HAS_DB` → **SqlDatabase** - Databases hosted on this server

## Examples

```cypher
// Find all SQL servers with public access enabled
MATCH (s:SqlServer)
WHERE s.publicNetworkAccess = "Enabled"
RETURN s.name, s.fullyQualifiedDomainName, s.version
```

```cypher
// Find SQL servers and their databases
MATCH (s:SqlServer)-[:HAS_DB]->(db:SqlDatabase)
RETURN s.name, collect(db.name) AS databases
```
