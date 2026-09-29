# CREATED

Links a creator application to an application or service principal using `createdByAppId`. The edge is created only when that property is present.

## Usage

- **GraphObject** → `CREATED` → **GraphApplication** - Creator matched by `appId`, stored in lowercase
- **GraphApplication** → `CREATED` → **GraphServicePrincipal** - Creator matched by `appId`

A creator can initially be a placeholder with only an `appId`; its full details need not have been collected.

## Properties

No additional properties on the relationship.

## Examples

```cypher
MATCH (creator)-[:CREATED]->(created)
RETURN creator.appId, created.displayName, labels(created)
```
