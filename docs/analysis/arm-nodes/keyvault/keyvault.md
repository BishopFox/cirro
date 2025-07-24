# KeyVault

Represents Azure Key Vault instances.

**Labels:** `:ArmResource:KeyVault`

**Properties:**

- `id` - Key Vault resource ID (primary key)
- `enableRbacAuthorization` - Whether RBAC authorization is enabled
- `enableSoftDelete` - Whether soft delete is enabled
- `enabledForDeployment` - Whether enabled for deployment
- `publicNetworkAccess` - Public network access setting
- `softDeleteRetentionInDays` - Soft delete retention period
- `vaultUri` - Key Vault URI

**Relationships:**
- `HAS_POLICY` → GraphObject (access policies for users/service principals)
