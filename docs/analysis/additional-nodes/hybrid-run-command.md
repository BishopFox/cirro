# HybridRunCommand

Represents a run command resource associated with an Azure Arc-enabled hybrid machine.

**Labels:** `:ArmResource:HybridRunCommand`

**Properties:**

- `id` - Run command resource ID (primary key)
- `asyncExecution` - Whether execution is asynchronous
- `runAsUser` - User context used to run the command
- `timeoutInSeconds` - Command timeout in seconds
- `treatFailureAsDeploymentFailure` - Failure handling behavior
- `script` - Script body
- `output` - Command output from the last execution

## Relationships

### Incoming

- **HybridMachine** → `HAS_RUN_COMMAND` → **HybridRunCommand** - Parent hybrid machine

### Outgoing

- **HybridRunCommand** → `HAS_PARAMETER` → **HybridRunCommandParameter** - Parameters bound to the run command

## Examples

```cypher
// Find hybrid run commands and their parameters
MATCH (hm:HybridMachine)-[:HAS_RUN_COMMAND]->(cmd:HybridRunCommand)
OPTIONAL MATCH (cmd)-[:HAS_PARAMETER]->(p:HybridRunCommandParameter)
RETURN hm.displayName, cmd.id, collect(p.name) AS parameterNames
```
