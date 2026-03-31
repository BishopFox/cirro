# Snapshot

Represents disk snapshots.

**Labels:** `:ArmResource:Snapshot`

**Properties:**

- `id` - Snapshot resource ID (primary key)
- `createOption` - How the snapshot was created
- `dataAccessAuthMode` - Data access authentication mode
- `diskSizeBytes` - Disk size in bytes
- `diskSizeGB` - Disk size in GB
- `diskState` - Disk state
- `hyperVGeneration` - Hyper-V generation
- `incremental` - Whether this is an incremental snapshot
- `networkAccessPolicy` - Network access policy
- `osType` - OS type
- `publicNetworkAccess` - Public network access setting

**Relationships:**
- `HAS_SNAPSHOT` → ArmResource (source resource)
