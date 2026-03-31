# AutomationRunbook

Represents Azure Automation runbooks.

**Labels:** `:ArmResource:AutomationRunbook`

**Properties:**

- `id` - Automation runbook resource ID (primary key)
- `creationTime` - Runbook creation timestamp
- `lastModifiedTime` - Last modification timestamp
- `description` - Runbook description
- `jobCount` - Number of jobs for this runbook
- `runbookType` - Type of runbook (e.g., PowerShell, Python)
- `state` - Current state of the runbook
- `runtimeEnvironment` - Runtime environment for the runbook

## Relationships

### Outgoing

- `HAS_PARAMETER` → **RunbookParameter** - Parameters defined for this runbook

### Incoming

- **AutomationAccount** → `HAS_RUNBOOK` → AutomationRunbook - Parent automation account

## Examples

```cypher
// Find all PowerShell runbooks
MATCH (rb:AutomationRunbook)
WHERE rb.runbookType = "PowerShell"
RETURN rb.name, rb.description, rb.state
```

```cypher
// Find runbooks with their parameters
MATCH (rb:AutomationRunbook)-[:HAS_PARAMETER]->(p:RunbookParameter)
RETURN rb.name, collect(p.name) AS parameters
```