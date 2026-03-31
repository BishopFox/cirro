# HAS_ROUTE

Represents the relationship between route tables and their individual routes.

## Usage

This relationship connects route tables to their configured routes:

- **RouteTable** → `HAS_ROUTE` → **NetworkRoute** - Route tables to their individual routes

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all route tables and their routes
MATCH (rt:RouteTable)-[:HAS_ROUTE]->(route:NetworkRoute)
RETURN rt.name, route.name, route.addressPrefix, route.nextHopType
```
