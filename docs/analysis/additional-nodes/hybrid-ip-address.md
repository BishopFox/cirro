# HybridIPAddress

Represents IP addresses associated with hybrid machines (not an ARM resource itself).

**Labels:** `:HybridIPAddress`

**Properties:**

- `id` - IP address ID (primary key)
- `ipAddress` - The IP address value
- `type` - IP address type (e.g., private, public)
- `subnetId` - Associated subnet ID if applicable
- `isPrimary` - Whether this is the primary IP address

## Relationships

### Incoming

- **HybridMachine** → `HAS_IP` → HybridIPAddress - Parent hybrid machine

## Examples

```cypher
// Find all hybrid machines and their IP addresses
MATCH (hm:HybridMachine)-[:HAS_IP]->(ip:HybridIPAddress)
RETURN hm.displayName, ip.ipAddress, ip.type, ip.isPrimary
```

```cypher
// Find hybrid machines with private IP addresses
MATCH (hm:HybridMachine)-[:HAS_IP]->(ip:HybridIPAddress)
WHERE ip.type = "private"
RETURN hm.displayName, ip.ipAddress
```