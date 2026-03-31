# VMRunCommand

Represents a run command resource associated with a virtual machine.

**Labels:** `:VMRunCommand`

**Properties:**

- `id` - Run command resource ID (primary key)
- `asyncExecution` - Whether execution is asynchronous
- `runAsUser` - User context used to run command
- `timeoutInSeconds` - Command timeout
- `treatFailureAsDeploymentFailure` - Failure handling behavior
- `script` - Script body

## Relationships

### Incoming

- **VirtualMachine** → `HAS_RUN_COMMAND` → **VMRunCommand** - VM run-command association

### Outgoing

- **VMRunCommand** → `HAS_PARAMETER` → **VMRunCommandParameter** - Parameters bound to the run command

## Examples

```cypher
// Find VM run commands and parameters
MATCH (vm:VirtualMachine)-[:HAS_RUN_COMMAND]->(cmd:VMRunCommand)
OPTIONAL MATCH (cmd)-[:HAS_PARAMETER]->(p:VMRunCommandParameter)
RETURN vm.name, cmd.id, collect(p.name) AS parameterNames
```
