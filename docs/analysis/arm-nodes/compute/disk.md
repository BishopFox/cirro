# Disk

Represents Azure managed disks.

**Labels:** `:ArmResource:Disk`

**Properties:**

- `id` - Disk resource ID (primary key)
- `managedBy` - ID of resource managing this disk
- `diskSizeGB` - Disk size in GB
- `diskState` - Current disk state
- `osType` - OS type (if OS disk)
- `networkAccessPolicy` - Network access policy
- `publicNetworkAccess` - Public network access setting

**Relationships:**
- `HAS_SNAPSHOT` ← Snapshot
