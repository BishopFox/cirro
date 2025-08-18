# ArcSqlDatabase

Azure Arc-enabled SQL Server database instances that extend Azure management to on-premises SQL Server databases.

**Labels:** `:ArmResource:ArcSqlDB`

## Properties

- `id` - Resource ID (inherited from ArmResource)
- `name` - Resource name (inherited from ArmResource)
- `type` - Resource type (inherited from ArmResource)
- `location` - Azure region (inherited from ArmResource)
- `tags` - Resource tags (inherited from ArmResource)
- `dataFileSizeMB` - Size of the database data file in MB
- `databaseCreationDate` - Date when the database was created
- `lastDatabaseUploadTime` - Last time database metadata was uploaded to Azure
- `isReadOnly` - Whether the database is read-only
- `state` - Current state of the database

## Relationships

- `ArcSqlServer` → `HAS_DB` → `ArcSqlDatabase` - The Arc SQL Server instance that hosts this database
