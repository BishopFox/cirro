# Tenant

Represents Azure tenants - the top-level organizational units in Azure.

**Labels:** `:Tenant`

**Properties:**

- `id` - Tenant ID (primary key)
- `displayName` - Tenant display name
- `tenantId` - Tenant identifier
- `countryCode` - Country code
- `domains` - Array of domains
- `defaultDomain` - Default domain
- `tenantCategory` - Tenant category
- `tenantType` - Tenant type

**Relationships:**
- `CONTAINS` → ManagementGroup
- `ASSOCIATED_WITH` → GraphOrg (bidirectional)
