# ContainerGroup

Represents an Azure Container Instances container group that hosts one or more containers.

**Labels:** `:ArmResource`, `:ContainerGroup`

**Properties:**

- `id` - Resource identifier (primary key)
- `nameservers` - DNS nameservers configuration

## Relationships

- **HAS_CONTAINER** → **Container** - Contains individual containers