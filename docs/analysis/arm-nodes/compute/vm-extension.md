# VMExtension

Represents virtual machine extensions (not an ARM resource itself).

**Labels:** `:VMExtension`

**Properties:**

- `id` - Extension ID (primary key)
- `name` - Extension name
- `type` - Extension type
- `location` - Extension location
- `provisioningState` - Provisioning state
- `settings` - Extension settings

**Relationships:**
- `HAS_EXTENSION` ← VirtualMachine
