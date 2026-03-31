# ApimService

Represents Azure API Management Service instances.

**Labels:** `:ArmResource:ApimService`

**Properties:**

- `id` - Resource ID (primary key)
- `createdAt` - Creation timestamp
- `developerPortalStatus` - Developer portal status
- `developerPortalUrl` - Developer portal URL
- `gatewayRegionalUrl` - Gateway regional URL
- `gatewayUrl` - Gateway URL
- `legacyPortalStatus` - Legacy portal status
- `managementApiUrl` - Management API URL
- `outboundPublicIpAddresses` - Outbound public IP addresses
- `portalUrl` - Portal URL
- `publicIpAddresses` - Public IP addresses
- `publicNetworkAccess` - Public network access setting
- `publisherEmail` - Publisher email
- `publisherName` - Publisher name
- `publisherChannel` - Publisher channel
- `scmUrl` - SCM URL

## Relationships

### Outgoing

- **Tenant** → `MANAGES` → **Tenant** - Managed by tenant relationship (from managed tenants)

## Examples

```cypher
// Find APIM services with public access
MATCH (a:ApimService)
WHERE a.publicNetworkAccess = 'Enabled'
RETURN a.name, a.gatewayUrl, a.publisherName
```
