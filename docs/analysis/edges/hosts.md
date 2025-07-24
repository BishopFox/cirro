# HOSTS

Represents hosting relationships between resources.

**Direction:** `(hostResource)-[:HOSTS]->(hostedResource)`

**Description:** Indicates that one resource hosts, contains, or provides the runtime environment for another resource.

**Common Patterns:**

- Container resources host application instances
- Server resources host database instances
- Infrastructure resources host workload resources
- Parent resources provide runtime environment for child resources

**Properties:** None

## Query Examples

```cypher
// Find all hosting relationships
MATCH path = (host)-[:HOSTS]->(hosted)
RETURN path

// Find resources hosting multiple other resources
MATCH (host)-[:HOSTS]->(hosted)
WITH host, COUNT(hosted) as hostedCount
WHERE hostedCount > 1
RETURN host, hostedCount
ORDER BY hostedCount DESC

// Find specific hosting relationships (e.g., servers hosting databases)
MATCH (host)-[:HOSTS]->(hosted)
WHERE 'Server' IN labels(host) AND 'Database' IN labels(hosted)
RETURN host.name, host.type, hosted.name, hosted.version, hosted.status

// Find unhosted resources
MATCH (resource)
WHERE NOT (resource)<-[:HOSTS]-()
  AND any(label IN labels(resource) WHERE label ENDS WITH 'Instance' OR label ENDS WITH 'Service')
RETURN resource

// Find hosting relationships by resource type
MATCH (host)-[:HOSTS]->(hosted)
RETURN labels(host) as hostType, labels(hosted) as hostedType, COUNT(*) as count
ORDER BY count DESC

// Find hosting chains (nested hosting)
MATCH path = (topHost)-[:HOSTS*1..3]->(resource)
RETURN path
```
