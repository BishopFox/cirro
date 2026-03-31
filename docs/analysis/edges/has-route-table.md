# HAS_ROUTE_TABLE

Represents the relationship between subnets and their associated route tables.

## Usage

This relationship connects subnets to their route tables:

- **Subnet** → `HAS_ROUTE_TABLE` → **RouteTable** - Subnets to their route tables

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all subnets and their route tables
MATCH (subnet:Subnet)-[:HAS_ROUTE_TABLE]->(rt:RouteTable)
RETURN subnet.name, rt.name
```
