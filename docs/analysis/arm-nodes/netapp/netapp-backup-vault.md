# NetAppBackupVault

Represents a backup vault for Azure NetApp Files account backups.

**Labels:** `:ArmResource`, `:NetAppBackupVault`

**Properties:**

- `id` - Resource identifier (primary key)
- `provisioningState` - Current provisioning state

## Relationships

- **NetAppAccount** → **HAS_BACKUP_VAULT** → **NetAppBackupVault** - Associated backup vault for the NetApp account
