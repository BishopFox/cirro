# HAS_ENVVAR

Represents environment variables attached to a container image definition.

## Usage

- **ContainerImage** → `HAS_ENVVAR` → **EnvVar**

## Examples

```cypher
MATCH (img:ContainerImage)-[:HAS_ENVVAR]->(e:EnvVar)
RETURN img.name, collect({name: e.name, value: e.value, secretRef: e.secretRef}) AS envs
```
