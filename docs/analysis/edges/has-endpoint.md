# HAS_ENDPOINT

Represents the relationship between Graph service principals and their endpoints.

## Usage

This relationship connects Graph service principals to their configured endpoints:

- **GraphServicePrincipal** → `HAS_ENDPOINT` → **ServicePrincipalEndpoint** - Service principals to their endpoints

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all service principals with endpoints
MATCH (sp:GraphServicePrincipal)-[:HAS_ENDPOINT]->(endpoint)
RETURN sp.displayName, endpoint.url, endpoint.type
```
