# HAS_RUNBOOK

Represents the relationship between automation accounts and their runbooks.

## Usage

This relationship connects automation accounts to their runbooks:

- **AutomationAccount** → `HAS_RUNBOOK` → **Runbook** - Automation accounts to their runbooks

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all automation accounts and their runbooks
MATCH (aa:AutomationAccount)-[:HAS_RUNBOOK]->(rb:Runbook)
RETURN aa.name, rb.name, rb.runbookType
```
