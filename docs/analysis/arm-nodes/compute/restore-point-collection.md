# RestorePointCollection

Represents VM restore point collections.

**Labels:** `:ArmResource:RestorePointCollection`

**Properties:**

- `id` - Restore point collection resource ID (primary key)
- `restorePointCollectionId` - Collection identifier used for VM correlation

**Relationships:**
- `HAS_RESTOREPOINTS` <- VirtualMachine (source VM)
- `HAS_INSTANCE` -> RestorePoint (instances in the collection)
