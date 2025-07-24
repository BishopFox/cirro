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

**Relationships:**
- `HAS_DB` ← SqlServer
