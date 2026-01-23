# NetAppSnapshotPolicy

Represents a snapshot policy for Azure NetApp Files volumes that defines automated backup schedules.

**Labels:** `:ArmResource`, `:NetAppSnapshotPolicy`

**Properties:**

- `id` - Resource identifier (primary key)
- `enabled` - Whether the policy is enabled
- `dailyScheduleHour` - Hour for daily snapshots
- `dailyScheduleMinute` - Minute for daily snapshots
- `dailyScheduleSnapshotsToKeep` - Number of daily snapshots to retain
- `hourlyScheduleMinute` - Minute for hourly snapshots
- `hourlyScheduleSnapshotsToKeep` - Number of hourly snapshots to retain
- `weeklyScheduleDay` - Day of week for weekly snapshots
- `weeklyScheduleHour` - Hour for weekly snapshots
- `weeklyScheduleMinute` - Minute for weekly snapshots
- `weeklyScheduleSnapshotsToKeep` - Number of weekly snapshots to retain
- `monthlyScheduleDaysOfMonth` - Days of month for monthly snapshots
- `monthlyScheduleHour` - Hour for monthly snapshots
- `monthlyScheduleMinute` - Minute for monthly snapshots
- `monthlyScheduleSnapshotsToKeep` - Number of monthly snapshots to retain

## Relationships

- **NetAppAccount** → **HAS_POLICY** → **NetAppSnapshotPolicy** - Associated with NetApp accounts