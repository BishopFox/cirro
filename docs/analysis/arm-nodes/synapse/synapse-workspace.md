# SynapseWorkspace

Represents Azure Synapse Analytics workspaces.

**Labels:** `:ArmResource:SynapseWorkspace`

**Properties:**

- `id` - Synapse workspace resource ID (primary key)
- `azureADOnlyAuthentication` - Whether Entra ID-only authentication is enabled
- `publicNetworkAccess` - Public network access setting
- `sqlAdministratorLogin` - SQL administrator login name
- `defaultDLSAccount` - Default Data Lake Storage account URL
- `defaultDLSFilesystem` - Default Data Lake Storage filesystem name

**Relationships:**
- `USES_STORAGE_DFS` → StorageAccountDFS (default data lake storage)
- Connected to Storage Account via StorageAccountDFS