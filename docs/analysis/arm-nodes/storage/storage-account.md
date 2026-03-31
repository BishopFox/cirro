# StorageAccount

Represents Azure storage accounts.

**Labels:** `:ArmResource:StorageAccount`

**Properties:**

- `id` - Storage account resource ID (primary key)
- `kind` - Storage account kind
- `accessTier` - Access tier (Hot, Cool, Archive)
- `primaryLocation` - Primary location
- `publicNetworkAccess` - Public network access setting
- `defaultToOAuthAuthentication` - Default to OAuth authentication
- `allowBlobPublicAccess` - Allow blob public access
- `allowCrossTenantReplication` - Allow cross-tenant replication
- `allowSharedKeyAccess` - Allow shared key access
- `largeFileSharesState` - Large file shares state
- `isLocalUserEnabled` - Local user enabled
- `isNfsV3Enabled` - NFSv3 enabled
- `isSftpEnabled` - SFTP enabled
- `supportsHttpsTrafficOnly` - HTTPS traffic only
- `minimumTlsVersion` - Minimum TLS version
- `networkAclBypass` - Network ACL bypass setting
- `networkAclDefaultAction` - Network ACL default action

**FileStorage-specific properties (when kind = FileStorage):**

- `defaultSharePermission` - Default share permission
- `directoryServiceOptions` - Directory service options
- `adDomainName` - Active Directory domain name
- `adDomainSid` - Active Directory domain SID
- `accountType` - AD account type
- `forestName` - AD forest name
- `netBiosDomainName` - NetBIOS domain name
- `samAccountName` - SAM account name

## Examples

```cypher
// Find storage accounts with public blob access allowed
MATCH (sa:StorageAccount)
WHERE sa.allowBlobPublicAccess = true
RETURN sa.name, sa.publicNetworkAccess, sa.accessTier
```

```cypher
// Find storage accounts by access tier
MATCH (sa:StorageAccount)
WHERE sa.accessTier = "Hot"
RETURN sa.name, sa.primaryLocation, sa.minimumTlsVersion
```
