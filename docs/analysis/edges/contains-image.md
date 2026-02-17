# CONTAINS_IMAGE

Represents images hosted by a Container Registry.

## Usage

- **ContainerRegistry** → `CONTAINS_IMAGE` → **ContainerImage**

## Examples

```cypher
MATCH (acr:ContainerRegistry)-[:CONTAINS_IMAGE]->(img:ContainerImage)
RETURN acr.loginServer, img.name
```
