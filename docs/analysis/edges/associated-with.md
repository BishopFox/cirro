# ASSOCIATED_WITH

Creates a bidirectional relationship between Tenant and GraphOrg nodes, linking Azure tenant information with Microsoft Graph organization data.

**Direction:** `(tenant:Tenant)-[:ASSOCIATED_WITH]->(org:GraphOrg)` and `(org:GraphOrg)-[:ASSOCIATED_WITH]->(tenant:Tenant)`

**Description:** This relationship establishes the connection between Azure tenant metadata (from ARM API) and organizational details (from Microsoft Graph API). The relationship is bidirectional, ensuring easy traversal in both directions during queries.

**Properties:** None

## Query Examples

```cypher
// Find the Graph organization associated with a tenant
MATCH (t:Tenant)-[:ASSOCIATED_WITH]->(org:GraphOrg)
WHERE t.tenantId = 'your-tenant-id'
RETURN org

// Find the tenant associated with a Graph organization  
MATCH (org:GraphOrg)-[:ASSOCIATED_WITH]->(t:Tenant)
WHERE org.id = 'your-org-id'
RETURN t

// Find all tenant-organization associations
MATCH path=(t:Tenant)-[:ASSOCIATED_WITH]-(org:GraphOrg)
RETURN path
```
