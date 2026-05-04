# OracleAutonomousDB

Represents Oracle Autonomous Database resources discovered from Azure ARM.

**Labels:** `:ArmResource:OracleAutonomousDB`

**Properties:**

- `id` - Resource ID (primary key)
- `actualUsedDataStorageSizeInTbs` - Actual used storage size in TB
- `allocatedStorageSizeInTbs` - Allocated storage size in TB
- `backupRententionPeriodInDays` - Backup retention period in days
- `characterSet` - Database character set
- `databaseType` - Oracle database type (from ARM `dataBaseType`)
- `dbVersion` - Database version
- `displayName` - Display name
- `isLocalDataGuardEnabled` - Whether local Data Guard is enabled
- `isMtlsConnectionRequired` - Whether mTLS is required for connections
- `isRemoteDataGuardEnabled` - Whether remote Data Guard is enabled
- `localDisasterRecoveryType` - Local disaster recovery type
- `ociUrl` - Oracle Cloud URL
- `ocid` - Oracle Cloud identifier
- `openMode` - Database open mode
- `permissionLevel` - Permission/access level
- `privateEndpoint` - Private endpoint resource
- `privateEndpointIp` - Private endpoint IP address
- `privateEndpointLabel` - Private endpoint label
- `timeCreated` - Creation timestamp
- `timeLocalDataGuardEnabled` - Timestamp when local Data Guard was enabled
- `highConnectionString` - High workload connection string
- `mediumConnectionString` - Medium workload connection string
- `lowConnectionString` - Low workload connection string
- Dynamic URL properties from `connectionUrls` (e.g. `apexUrl`, `ordsUrl`, `sqlDevWebUrl`, `graphStudioUrl`, `machineLearningNotebookUrl`, `databaseTransformsUrl`)

## Relationships

### Incoming

- **ArmResource** → `HAS_ORACLE_RESOURCE` → **OracleAutonomousDB** - Subnet-linked parent resource contains this database

## Examples

```cypher
// Find Oracle autonomous databases and connection info
MATCH (db:OracleAutonomousDB)
RETURN db.displayName, db.dbVersion, db.openMode, db.permissionLevel
```

```cypher
// Find databases with mTLS not required
MATCH (db:OracleAutonomousDB)
WHERE db.isMtlsConnectionRequired = false
RETURN db.displayName, db.privateEndpoint
```
