# MLWorkspace

Represents Azure Machine Learning Workspaces.

**Labels:** `:ArmResource:MLWorkspace`

**Properties:**

- `id` - Resource ID (primary key)
- `allowRoleAssignmentOnRG` - Allow role assignment on resource group
- `creationTime` - Creation time
- `discoveryUrl` - Discovery URL
- `friendlyName` - Friendly name
- `networkAclBypass` - Network ACL bypass setting
- `networkAclDefaultAction` - Network ACL default action
- `notebookInfoFqdn` - Notebook info FQDN
- `isPrivateLinkEnabled` - Whether private link is enabled
- `publicNetworkAccess` - Public network access setting
- `softDeleteEnabled` - Whether soft delete is enabled
- `tenantId` - Tenant ID

## Examples

```cypher
// Find ML workspaces with public network access
MATCH (ml:MLWorkspace)
WHERE ml.publicNetworkAccess = 'Enabled'
RETURN ml.name, ml.friendlyName
```
