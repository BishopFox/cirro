# EnvVar

Represents an environment variable defined for a container in a Container App template.

**Labels:** `:EnvVar`

**Properties:**

- `name` - Variable name (primary key per image edge)
- `value` - Literal value (if provided)
- `secretRef` - Secret reference name (if value is sourced from a secret)

## Relationships

### Incoming
- **ContainerImage** → `HAS_ENVVAR` → **EnvVar** - Environment variable attached to an image definition

## Examples

```cypher
// Find env vars that reference secrets
MATCH (img:ContainerImage)-[:HAS_ENVVAR]->(e:EnvVar)
WHERE e.secretRef IS NOT NULL
RETURN img.name, e.name, e.secretRef
```
