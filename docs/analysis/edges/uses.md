# USES

Represents resource usage relationships.

**Direction:** `(resource)-[:USES]->(usedResource)`

**Description:** Indicates that one resource uses or depends on another resource.

**Common Patterns:**
- Virtual machines use disks
- Virtual machines use availability sets

**Properties:** None

## Query Examples

```cypher
// Find all disks used by virtual machines
MATCH path=(vm:VirtualMachine)-[:USES]->(disk:Disk)
RETURN path

// Find resources using managed identities
MATCH path=(resource)-[:USES]->(identity:UserAssignedManagedIdentity)
RETURN path

// Find resource dependencies
MATCH path=(resource)-[:USES]->(dependency)
RETURN path
```
