# REGISTERED_USER

Represents device registration relationships.

**Direction:** `(user)-[:REGISTERED_USER]->(device)`

**Description:** Indicates that a user is registered as a user of a device.

**Properties:** None

## Query Examples

```cypher
// Find all devices registered to a user
MATCH path=(u:GraphUser)-[:REGISTERED_USER]->(d:GraphDevice)
RETURN path

// Find users registered to a specific device
MATCH path=(u:GraphUser)-[:REGISTERED_USER]->(d:GraphDevice {displayName: 'LAPTOP-001'})
RETURN path

// Find devices with multiple registered users
MATCH (u:GraphUser)-[:REGISTERED_USER]->(d:GraphDevice)
WITH d, COUNT(u) as userCount
WHERE userCount > 1
RETURN d, userCount
```
