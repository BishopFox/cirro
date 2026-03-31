# GraphOrg

Represents Entra ID organizations collected from Microsoft Graph. This node provides detailed information about the organization/tenant structure within Azure Active Directory.

**Labels:** `:GraphOrg`

**Properties:**

- `id` - Organization ID (primary key)
- `businessPhones` - Array of organization's business phone numbers
- `createdDateTime` - When the organization was created
- `displayName` - Organization's display name
- `modifiedDateTime` - When the organization was last modified
- `onPremisesLastSyncDateTime` - Last synchronization time with on-premises
- `onPremisesSyncEnabled` - Whether on-premises sync is enabled
- `tenantType` - Type of tenant (e.g., AAD, AAD B2C)


**Notes:**
- GraphOrg nodes are automatically linked to their corresponding Tenant nodes via bidirectional ASSOCIATED_WITH relationships
- Each organization can have multiple verified domains
- This node type was introduced to provide richer organizational metadata from Microsoft Graph API
