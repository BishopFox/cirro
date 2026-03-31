# Container

Represents an individual container within an Azure Container Instances container group.

**Labels:** `:Container`

**Properties:**

- `image` - Container image name (primary key)
- `name` - Container name
- `environmentVariables` - List of environment variables in "name:value" format

## Relationships

- **ContainerGroup** → **HAS_CONTAINER** → **Container** - Container within container groups