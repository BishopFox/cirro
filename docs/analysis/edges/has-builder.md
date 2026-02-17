# HAS_BUILDER

Associates a Container App (or related resource) with its App Builder resource.

## Usage

- **AppBuilder** relationships are not currently emitted, but this edge is reserved for linking builder resources when available.

## Examples

```cypher
// Placeholder until builder relationships are emitted
MATCH (:AppBuilder)-[r:HAS_BUILDER]->(n)
RETURN count(r) AS edges
```
