# NetAppAccount

Represents an Azure NetApp Files account for managing file storage pools and volumes.

**Labels:** `:ArmResource`, `:NetAppAccount`

**Properties:**

- `id` - Resource identifier (primary key)
- `multiADStatus` - Status of multi-Active Directory configuration
- `provisioningState` - Current provisioning state
- `encryptionKeySource` - Source of encryption keys

## Relationships

- **IS_DEPLOYED** → **NetAppAccountADDomain** - Links to Active Directory domain configurations
- **CONTAINS** → **NetAppCapacityPool** - Contains capacity pools
- **HAS_BACKUP_VAULT** → **NetAppBackupVault** - Associated backup vaults
- **HAS_POLICY** → **NetAppSnapshotPolicy** - Associated snapshot policies