# VMRunCommandParameter

Represents a parameter for a virtual machine run command.

**Labels:** `:VMRunCommandParameter`

**Properties:**

- `name` - Parameter name (composite key with `commandId`)
- `commandId` - Parent run command ID (composite key)
- `value` - Parameter value

## Relationships

- **VMRunCommand** → **HAS_PARAMETER** → **VMRunCommandParameter** - Parameters for run commands