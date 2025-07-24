# HybridExtension

Represents extensions installed on hybrid machines (not an ARM resource itself).

**Labels:** `:HybridExtension`

**Properties:**

- `id` - Extension ID (primary key)
- `name` - Extension name
- `type` - Extension type
- `location` - Extension location
- `typeHandlerVersion` - Type handler version
- `autoUpgradeMinorVersion` - Auto upgrade minor version
- `enableAutomaticUpgrade` - Enable automatic upgrade
- `statusMessage` - Status message
- `provisioningState` - Provisioning state

**Relationships:**
- `HAS_EXTENSION` ← HybridMachine
