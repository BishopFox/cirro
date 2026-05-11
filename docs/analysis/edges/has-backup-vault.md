# HAS_BACKUP_VAULT

Links NetApp accounts to their associated NetApp backup vault resources.

**Relationship:** `:HAS_BACKUP_VAULT`

**Direction:** `(:NetAppAccount)-[:HAS_BACKUP_VAULT]->(:NetAppBackupVault)`

## Description

This relationship indicates which backup vault belongs to a specific NetApp account.

## Examples

### NetAppAccount → NetAppBackupVault
Connects an Azure NetApp Files account to the backup vault used to store and manage backups for that account.

## Use Cases

- Identifying backup coverage for NetApp accounts
- Mapping backup dependencies for NetApp storage workloads
- Tracing account-to-vault backup topology
