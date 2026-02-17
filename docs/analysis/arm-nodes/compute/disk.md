# Disk

Represents Azure managed disks.

**Labels:** `:ArmResource:Disk`

**Properties:**

- `id` - Disk resource ID (primary key)
- `diskSizeGB` - Disk size in GB
- `diskState` - Current disk state
- `osType` - OS type (if OS disk)
- `hyperVGeneration` - Hyper-V generation
- `networkAccessPolicy` - Network access policy
- `publicNetworkAccess` - Public network access setting
- `timeCreated` - Creation timestamp
- `uniqueId` - Unique identifier
- `fromImage` - Source image ID when created FromImage
- `sourceUri` - Source URI or resource when imported/copied

**Relationships:**
- `HAS_SNAPSHOT` ← Snapshot
- `HAS_SOURCE` ← StorageAccount - Source storage for Import/Copy operations
- `ENCRYPTED_BY` → DiskEncryptionSet - Encryption configuration
