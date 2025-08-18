# WebSite

Represents Azure App Service web apps.

**Labels:** `:ArmResource:WebSite`

**Properties:**

- `id` - Web site resource ID (primary key)
- `defaultHostName` - Default host name
- `enabledHostNames` - Array of enabled host names
- `repositorySiteName` - Repository site name
- `usageState` - Usage state
- `enabled` - Whether the site is enabled
- `isReserved` - Whether the site is reserved
- `isXenon` - Whether the site is Xenon
- `hyperV` - Whether Hyper-V is used
- `vnetImagePullEnabled` - Whether VNet image pull is enabled
- `vnetContentShareEnabled` - Whether VNet content share is enabled
- `vnetBackupRestoreEnabled` - Whether VNet backup restore is enabled
- `lastModifiedTimeUtc` - Last modified time
- `storageRecoveryDefaultState` - Storage recovery default state
- `contentAvailabilityState` - Content availability state
- `runtimeAvailabilityState` - Runtime availability state
- `secretsCollection` - Secrets collection
- `vnetRouteAllEnabled` - Whether VNet route all is enabled
- `httpsOnly` - Whether HTTPS only is enforced
- `redundancyMode` - Redundancy mode
- `inProgressOperationId` - In-progress operation ID

**Relationships:**
- `HOSTS_SITE` ← ServerFarm
