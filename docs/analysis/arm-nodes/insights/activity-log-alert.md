# ActivityLogAlert

Represents Azure Monitor Activity Log Alerts.

**Labels:** `:ArmResource:ActivityLogAlert`

**Properties:**

- `id` - Resource ID (primary key)
- `description` - Alert description
- `enabled` - Whether the alert is enabled
- `condition` - Alert condition (JSON)

## Relationships

### Incoming

- **ArmResource** → `HAS_ALERT` → **ActivityLogAlert** - Any ARM resource in scope fires this alert

## Examples

```cypher
// Find all enabled alerts and their scoped resources
MATCH (r:ArmResource)-[:HAS_ALERT]->(a:ActivityLogAlert)
WHERE a.enabled = true
RETURN r.id, labels(r), a.id, a.description
```
