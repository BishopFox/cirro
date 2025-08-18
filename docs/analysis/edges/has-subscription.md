# HAS_SUBSCRIPTION

Represents the relationship between management groups and their contained subscriptions.

## Usage

This relationship connects management groups to the Azure subscriptions they contain:

- **ManagementGroup** → `HAS_SUBSCRIPTION` → **Subscription** - Management groups containing subscriptions

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all subscriptions in a management group
MATCH (mg:ManagementGroup)-[:HAS_SUBSCRIPTION]->(sub:Subscription)
RETURN mg.displayName, sub.displayName

// Find management groups with many subscriptions
MATCH (mg:ManagementGroup)-[:HAS_SUBSCRIPTION]->(sub:Subscription)
WITH mg, COUNT(sub) as subCount
WHERE subCount > 5
RETURN mg.displayName, subCount
ORDER BY subCount DESC

// Find subscription hierarchy through management groups
MATCH path = (root:ManagementGroup)-[:HAS_ENTITY*]->(mg:ManagementGroup)-[:HAS_SUBSCRIPTION]->(sub:Subscription)
WHERE NOT (root)<-[:HAS_ENTITY]-(:ManagementGroup)
RETURN path
```
