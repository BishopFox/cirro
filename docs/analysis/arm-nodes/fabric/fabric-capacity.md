# FabricCapacity

Represents Microsoft Fabric capacities.

**Labels:** `:ArmResource:FabricCapacity`

**Properties:**

- `id` - Resource ID (primary key)
- `members` - Administration members list

## Examples

```cypher
MATCH (c:FabricCapacity)
RETURN c.id, c.members
```
