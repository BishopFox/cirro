# ContainerImage

Represents a container image reference used by Container Apps (including tag/digest in the name).

**Labels:** `:ContainerImage`

**Properties:**

- `name` - Full image name (including registry prefix and tag/digest)

## Relationships

### Incoming
- **ContainerRegistry** → `CONTAINS_IMAGE` → **ContainerImage** - Registry that hosts the image reference

### Outgoing
- **ContainerImage** → `HAS_ENVVAR` → **EnvVar** - Environment variables defined for the image in a Container App template

## Examples

```cypher
// List images and their registries
MATCH (acr:ContainerRegistry)-[:CONTAINS_IMAGE]->(img:ContainerImage)
RETURN acr.loginServer, img.name
```
