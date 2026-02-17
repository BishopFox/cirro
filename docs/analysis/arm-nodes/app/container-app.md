# ContainerApp

Represents Azure Container Apps.

**Labels:** `:ArmResource:ContainerApp`

**Properties:**

- `id` - Resource ID (primary key)
- `runningStatus` - Current running status
- `allowInsecure` - Allow insecure ingress
- `fqdn` - Fully qualified domain name
- `clientCertificateMode` - Client certificate mode for ingress
- `external` - External ingress enabled
- `exposedPort` - Exposed ingress port
- `targetPort` - Target application port
- `secretNames` - Secret names referenced by the app

## Relationships

### Incoming
- **AppManagedEnvironment** → `DEPLOYS` → **ContainerApp** - Deployment into a managed environment

### Outgoing
- **ContainerApp** → `CAN_ACCESS_REGISTRY` → **ContainerRegistry** - Registry access for image pulls (with identity/secret details)
- **ContainerRegistry** → `CONTAINS_IMAGE` → **ContainerImage** - Images referenced by this app (by registry server and image string)
- **ContainerImage** → `HAS_ENVVAR` → **EnvVar** - Environment variables defined on containers

## Examples

```cypher
// Container Apps and their registries
MATCH (app:ContainerApp)-[r:CAN_ACCESS_REGISTRY]->(acr:ContainerRegistry)
RETURN app.fqdn, acr.loginServer, r.identity, r.username, r.passwordSecretRef
```

```cypher
// Container Apps with images and env vars
MATCH (acr:ContainerRegistry)-[:CONTAINS_IMAGE]->(img:ContainerImage)-[:HAS_ENVVAR]->(e:EnvVar)
RETURN acr.loginServer, img.name, collect({name: e.name, value: e.value, secretRef: e.secretRef}) AS env
```
