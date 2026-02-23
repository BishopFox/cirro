# StaticWebsite

Represents Azure Static Web App resources.

**Labels:** `:ArmResource:StaticWebsite`

**Properties:**

- `id` - Resource ID (primary key)
- `allowConfigFileUpdates` - Whether config file updates are allowed
- `branch` - Source branch
- `contentDistributionEndpoint` - Content distribution endpoint
- `customDomains` - Custom domains
- `defaultHostName` - Default host name
- `deploymentAuthPolicy` - Deployment auth policy
- `keyVaultReferenceIdentity` - Key Vault reference identity
- `prettyName` - Pretty name
- `prettyNameHostname` - Pretty name hostname
- `prettyNameStatus` - Pretty name status
- `provider` - Provider
- `publicNetworkAccess` - Public network access setting
- `repositoryUrl` - Source repository URL
- `stableInboundIP` - Stable inbound IP address
- `stagingEnvironmentPolicy` - Staging environment policy

## Examples

```cypher
// Find static websites with their repository URLs
MATCH (sw:StaticWebsite)
RETURN sw.name, sw.defaultHostName, sw.repositoryUrl, sw.branch
```
