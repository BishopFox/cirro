# HAS_SOURCE

Represents the relationship between Event Grid resources and their source resources.

## Usage

This relationship connects Event Grid topics to their source Azure resources:

- **EventGridTopic** → `HAS_SOURCE` → **ArmResource** - Event Grid topics to their source resources

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all Event Grid topics and their source resources
MATCH (topic:EventGridTopic)-[:HAS_SOURCE]->(source:ArmResource)
RETURN topic.name, source.name, source.type
```
