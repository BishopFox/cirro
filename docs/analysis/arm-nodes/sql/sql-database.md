# SqlDatabase

Represents Azure SQL databases.

**Labels:** `:ArmResource:SqlDatabase`

**Properties:**

- `id` - SQL database resource ID (primary key)
- `collation` - Database collation
- `creationDate` - Database creation date
- `currentServiceObjectiveName` - Current service objective name
- `databaseId` - Database ID
- `isInfraEncryptionEnabled` - Infrastructure encryption enabled
- `maxSizeBytes` - Maximum size in bytes
- `status` - Database status

## Relationships

### Incoming

- **SqlServer** → `HAS_DB` → **SqlDatabase** - Parent SQL server

## Examples

```cypher
// Find databases by service tier
MATCH (db:SqlDatabase)
WHERE db.currentServiceObjectiveName = "S0"
RETURN db.name, db.maxSizeBytes, db.status
```

```cypher
// Find databases with encryption enabled
MATCH (db:SqlDatabase)
WHERE db.isInfraEncryptionEnabled = true
RETURN db.name, db.collation, db.creationDate
```
