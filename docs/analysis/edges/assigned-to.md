# ASSIGNED_TO

Represents general assignment relationships.

**Direction:** `(principal)-[:ASSIGNED_TO]->(resource/role)`

**Description:** Generic relationship indicating that a principal is assigned to a resource or role.

**Properties:** Varies based on assignment type

## Query Examples

```cypher
// Find all assignments for a principal
MATCH path=(principal)-[assignment:ASSIGNED_TO]->(target)
RETURN path

// Find resources with assignments
MATCH path=(principal)-[assignment:ASSIGNED_TO]->(resource)
RETURN path

// Find assignment patterns
MATCH (principal)-[assignment:ASSIGNED_TO]->(target)
WITH labels(target) as targetTypes, COUNT(*) as count
RETURN targetTypes, count
ORDER BY count DESC
```
