# VMRunCommandParameter

Represents a parameter for a virtual machine run command.

**Labels:** `:VMRunCommandParameter`

**Properties:**

- `name` - Parameter name (primary key)
- `value` - Parameter value

## Relationships

- **VMRunCommand** → **HAS_PARAMETER** → **VMRunCommandParameter** - Parameters for run commands