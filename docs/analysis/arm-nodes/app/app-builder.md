# AppBuilder

Represents Azure App Builders resources.

**Labels:** `:ArmResource:AppBuilder`

**Properties:**

- `id` - Resource ID (primary key)

## Examples

```cypher
// List App Builders
MATCH (b:AppBuilder)
RETURN b.id
```
