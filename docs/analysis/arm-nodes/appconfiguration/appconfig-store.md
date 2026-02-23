# AppConfigurationStore

Represents Azure App Configuration Stores.

**Labels:** `:ArmResource:AppConfigurationStore`

**Properties:**

- `id` - Resource ID (primary key)
- `creationDate` - Creation date
- `endpoint` - Configuration store endpoint
- `disableLocalAuth` - Whether local authentication is disabled
- `enablePurgeProtection` - Whether purge protection is enabled
- `softDeleteRetentionInDays` - Soft delete retention period in days
- `defaultKeyValueRevisionRetentionPeriodInSeconds` - Default key-value revision retention period in seconds

## Examples

```cypher
// Find App Configuration stores with local auth enabled
MATCH (ac:AppConfigurationStore)
WHERE ac.disableLocalAuth = false OR ac.disableLocalAuth IS NULL
RETURN ac.name, ac.endpoint
```
