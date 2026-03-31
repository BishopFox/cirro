# CONTAINS

Represents hierarchical containment relationships in Azure.

**Direction:** `(parent)-[:CONTAINS]->(child)`

**Description:** Indicates that one Azure resource contains or manages another resource.

**Common Patterns:**

- Tenants contain management groups
- Management groups contain subscriptions
- Subscriptions contain resource groups
- Resource groups contain resources
- Virtual networks contain subnets

**Properties:** None

## Query Examples

```cypher
// Find all resources in a resource group
MATCH path=(rg:ResourceGroup)-[:CONTAINS]->(resource)
RETURN path

// Find the full hierarchy from tenant to resource
MATCH path=(t:Tenant)-[:CONTAINS*]->(resource)
WHERE NOT (resource)-[:CONTAINS]->()
RETURN path

// Find empty resource groups
MATCH (rg:ResourceGroup)
WHERE NOT (rg)-[:CONTAINS]->()
RETURN rg
```
