# VMRunCommand

Represents a run command associated with a virtual machine for executing scripts and commands.

**Labels:** `:ArmResource`, `:VMRunCommand`

**Properties:**

- `id` - Resource identifier (primary key)
- `asyncExecution` - Whether the command executes asynchronously
- `runAsUser` - User context for command execution
- `timeoutInSeconds` - Command execution timeout
- `treatFailureAsDeploymentFailure` - Whether command failure should be treated as deployment failure
- `script` - Script content to execute

## Relationships

- **VirtualMachine** → **HAS_RUN_COMMAND** → **VMRunCommand** - Associated with virtual machines
- **HAS_PARAMETER** → **VMRunCommandParameter** - Contains command parameters