# AutomationAccount

Represents Azure Automation accounts.

**Labels:** `:ArmResource:AutomationAccount`

**Properties:**

- `id` - Automation account resource ID (primary key)
- `creationTime` - Account creation timestamp
- `lastModifiedTime` - Last modification timestamp
- `registrationUrl` - Registration URL for the automation account
- `automationHybridServiceUrl` - Automation hybrid service URL

## Relationships

### Outgoing

- **AutomationAccount** → `HAS_RUNBOOK` → **AutomationRunbook** - Runbooks in this automation account

## Examples

```cypher
// Find all automation accounts
MATCH (aa:AutomationAccount)
RETURN aa.name, aa.creationTime, aa.registrationUrl
```

```cypher
// Find automation accounts and their runbooks
MATCH (aa:AutomationAccount)-[:HAS_RUNBOOK]->(rb:AutomationRunbook)
RETURN aa.name, rb.name, rb.runbookType
```
