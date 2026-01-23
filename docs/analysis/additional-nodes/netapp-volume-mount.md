# NetAppVolumeMount

Represents a mount target for an Azure NetApp Files volume.

**Labels:** `:NetAppVolumeMount`

**Properties:**

- `mountTargetId` - Mount target identifier (primary key)
- `fileSystemId` - File system identifier
- `ipAddress` - IP address for mounting
- `smbServerFqdn` - SMB server fully qualified domain name

## Relationships

- **NetAppVolume** → **MOUNTS** → **NetAppVolumeMount** - Mount targets for volumes