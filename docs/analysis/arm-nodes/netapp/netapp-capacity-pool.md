# NetAppCapacityPool

Represents a capacity pool within an Azure NetApp Files account for managing storage capacity and performance tiers.

**Labels:** `:ArmResource`, `:NetAppCapacityPool`

**Properties:**

- `id` - Resource identifier (primary key)
- `coolAccess` - Whether cool access tier is enabled
- `encryptionType` - Type of encryption applied
- `poolId` - Unique pool identifier
- `qosType` - Quality of Service type (Auto/Manual)
- `size` - Size of the capacity pool in bytes

## Relationships

- **NetAppAccount** → **CONTAINS** → **NetAppCapacityPool** - Contained within NetApp accounts
- **CONTAINS** → **NetAppVolume** - Contains volumes