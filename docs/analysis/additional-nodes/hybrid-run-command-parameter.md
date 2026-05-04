# HybridRunCommandParameter

Represents a parameter for a hybrid machine run command.

**Labels:** `:HybridRunCommandParameter`

**Properties:**

- `name` - Parameter name (composite key with `commandId`)
- `commandId` - Parent run command ID (composite key)
- `value` - Parameter value

## Relationships

### Incoming

- **HybridRunCommand** → `HAS_PARAMETER` → **HybridRunCommandParameter** - Parent run command
