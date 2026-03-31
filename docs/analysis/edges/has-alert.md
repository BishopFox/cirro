# HAS_ALERT

Represents the relationship between an Azure resource scope and an Activity Log Alert that monitors it.

**Direction:**
- `(resource:ArmResource)-[:HAS_ALERT]->(alert:ActivityLogAlert)`

**Description:** Any Azure resource listed as a scope for an Activity Log Alert has a `HAS_ALERT` edge pointing to that alert. This relationship allows you to trace which resources are being monitored by alerting rules and identify gaps in alert coverage.

**Common Patterns:**
- Subscriptions, resource groups, or individual resources can be scopes for activity log alerts
- A single alert may monitor multiple scopes, resulting in multiple `HAS_ALERT` edges pointing to the same alert node
- Activity log alerts can be enabled or disabled; the `enabled` property on the `ActivityLogAlert` node reflects current state

**Properties:** None

## Query Examples

```cypher
// Find all enabled activity log alerts and the resources they monitor
MATCH (resource:ArmResource)-[:HAS_ALERT]->(alert:ActivityLogAlert)
WHERE alert.enabled = true
RETURN resource.id AS monitoredResource, alert.id AS alert, alert.description

// Find resources with no activity log alerts
MATCH (resource:ArmResource)
WHERE NOT (resource)-[:HAS_ALERT]->(:ActivityLogAlert)
RETURN resource.id

// Find all scopes for a specific alert
MATCH (resource:ArmResource)-[:HAS_ALERT]->(alert:ActivityLogAlert)
WHERE alert.id = '<alert-id>'
RETURN resource.id
```
