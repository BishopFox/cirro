# HAS_ENDPOINT

Represents the relationship between resources and their endpoints.

## Usage

This relationship connects resources to their configured endpoints:

- **GraphServicePrincipal** → `HAS_ENDPOINT` → **ServicePrincipalEndpoint** - Service principals to their endpoints
- **CdnProfile** → `HAS_ENDPOINT` → **AfdEndpoint** - CDN profiles to their Azure Front Door endpoints

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all service principals with endpoints
MATCH (sp:GraphServicePrincipal)-[:HAS_ENDPOINT]->(endpoint)
RETURN sp.displayName, endpoint.url, endpoint.type

// Find all CDN profiles with AFD endpoints
MATCH (cdn:CdnProfile)-[:HAS_ENDPOINT]->(endpoint:AfdEndpoint)
RETURN cdn.name, endpoint.hostName, endpoint.enabledState
```
