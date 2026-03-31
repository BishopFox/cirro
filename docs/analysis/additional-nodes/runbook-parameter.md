# RunbookParameter

Represents parameters defined for automation runbooks (not an ARM resource itself).

**Labels:** `:RunbookParameter`

**Properties:**

- `id` - Parameter ID (primary key)
- `name` - Parameter name
- `defaultValue` - Default value for the parameter
- `isMandatory` - Whether the parameter is mandatory
- `position` - Parameter position
- `type` - Parameter type

## Relationships

### Incoming

- **AutomationRunbook** → `HAS_PARAMETER` → RunbookParameter - Parent runbook

## Examples

```cypher
// Find all mandatory parameters
MATCH (p:RunbookParameter)
WHERE p.isMandatory = true
RETURN p.name, p.type, p.defaultValue
```

```cypher
// Find runbooks with the most parameters
MATCH (rb:AutomationRunbook)-[:HAS_PARAMETER]->(p:RunbookParameter)
RETURN rb.name, count(p) AS parameterCount
ORDER BY parameterCount DESC
```