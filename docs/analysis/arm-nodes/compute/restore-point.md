# RestorePoint

Represents individual restore point instances inside a restore point collection.

**Labels:** `:ArmResource:RestorePoint`

**Properties:**

- `id` - Restore point ARM resource ID (primary key)
- `consistencyMode` - Snapshot consistency mode
- `timeCreated` - Time the restore point was created

**Relationships:**
- `HAS_INSTANCE` <- RestorePointCollection (parent collection)
