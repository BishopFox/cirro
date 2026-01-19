# EventGridSystemTopic

Represents Azure Event Grid system topics.

**Labels:** `:ArmResource:EventGridTopic`

**Properties:**

- `id` - Event Grid system topic resource ID (primary key)
- `metricResourceId` - Metric resource identifier
- `source` - Source of the system topic
- `topicType` - Type of the system topic

## Relationships

### Outgoing

- **EventGridSystemTopic** → `HAS_SOURCE` → **ArmResource** - Source resource for the system topic

## Examples

```cypher
// Find all Event Grid system topics
MATCH (topic:EventGridTopic)
WHERE topic.topicType IS NOT NULL
RETURN topic.name, topic.source, topic.topicType
```

```cypher
// Find system topics and their source resources
MATCH (topic:EventGridTopic)-[:HAS_SOURCE]->(source:ArmResource)
RETURN topic.name, topic.topicType, source.name
```