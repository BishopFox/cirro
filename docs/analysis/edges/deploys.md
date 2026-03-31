# DEPLOYS

Represents deployment of Container Apps into a managed environment.

## Usage

- **AppManagedEnvironment** → `DEPLOYS` → **ContainerApp**

## Examples

```cypher
MATCH (env:AppManagedEnvironment)-[:DEPLOYS]->(app:ContainerApp)
RETURN env.defaultDomain, collect(app.fqdn) AS apps
```
