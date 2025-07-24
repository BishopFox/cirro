# REGISTERED_OWNER

Represents device ownership relationships.

**Direction:** `(user)-[:REGISTERED_OWNER]->(device)`

**Description:** Indicates that a user is registered as an owner of a device.

**Properties:** None

## Query Examples

```cypher
// Find all devices owned by a user
MATCH path=(u:GraphUser)-[:REGISTERED_OWNER]->(d:GraphDevice)
RETURN path

// Find the owner of a specific device
MATCH path=(u:GraphUser)-[:REGISTERED_OWNER]->(d:GraphDevice {displayName: 'LAPTOP-001'})
RETURN path

// Find devices without registered owners
MATCH (d:GraphDevice)
WHERE NOT (d) <-[:REGISTERED_OWNER]- ()
RETURN d
```
