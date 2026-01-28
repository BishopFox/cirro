# VERIFIED_DOMAIN

Connects GraphOrg nodes to their VerifiedDomain nodes, representing the domains that have been verified as belonging to the organization.

**Direction:** `(org:GraphOrg)-[:VERIFIED_DOMAIN]->(domain:VerifiedDomain)`

**Description:** This relationship links Entra ID organizations to their verified domains. Verified domains are important for understanding email routing, user identity, and organizational boundaries within Entra ID.

**Properties:** None

## Query Examples

```cypher
// Find all verified domains for an organization
MATCH (org:GraphOrg)-[:VERIFIED_DOMAIN]->(domain:VerifiedDomain)
WHERE org.displayName = 'Contoso'
RETURN domain

// Find the default domain for an organization
MATCH (org:GraphOrg)-[:VERIFIED_DOMAIN]->(domain:VerifiedDomain)
WHERE org.id = 'your-org-id' AND domain.isDefault = true
RETURN domain.name

// Find all organizations using a specific domain
MATCH (org:GraphOrg)-[:VERIFIED_DOMAIN]->(domain:VerifiedDomain)
WHERE domain.name CONTAINS 'contoso.com'
RETURN org
```
