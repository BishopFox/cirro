# NetAppVolume

Represents a volume within an Azure NetApp Files capacity pool.

**Labels:** `:ArmResource`, `:NetAppVolume`

**Properties:**

- `id` - Resource identifier (primary key)
- `baremetalTenantId` - Bare metal tenant identifier
- `coolAccess` - Cool access configuration
- `creationToken` - Unique creation token for the volume
- `encryptionKeySource` - Source of encryption keys
- `fileSystemId` - File system identifier
- `isDefaultQuotaEnabled` - Whether default quota is enabled
- `isLargeVolume` - Whether this is a large volume
- `kerberosEnabled` - Whether Kerberos authentication is enabled
- `ldapEnabled` - Whether LDAP is enabled
- `maximumNumberOfFiles` - Maximum number of files allowed
- `protocolTypes` - Supported protocol types (NFSv3, NFSv4.1, SMB, etc.)
- `securityStyle` - Security style (Unix/NTFS/Mixed)
- `smbAccessBasedEnumeration` - SMB access-based enumeration setting
- `smbContinuouslyAvailable` - SMB continuously available setting
- `smbEncryption` - SMB encryption setting
- `smbNonBrowsable` - SMB non-browsable setting
- `snapshotDirectoryVisible` - Whether snapshot directory is visible
- `usageThreshold` - Usage threshold in bytes
- `volumeType` - Type of volume

## Relationships

- **NetAppCapacityPool** → **CONTAINS** → **NetAppVolume** - Contained within capacity pools
- **MOUNTS** → **NetAppVolumeMount** - Associated mount targets