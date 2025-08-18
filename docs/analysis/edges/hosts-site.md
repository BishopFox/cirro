# HOSTS_SITE

Represents the relationship between server farms and the web sites they host.

## Usage

This relationship connects App Service plans (server farms) to the web applications they host:

- **ServerFarm** → `HOSTS_SITE` → **WebSite** - Server farms to their hosted web sites

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all server farms and the sites they host
MATCH (sf:ServerFarm)-[:HOSTS_SITE]->(site:WebSite)
RETURN sf.name, site.name, site.kind
```
