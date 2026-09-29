# GraphServicePrincipal

Represents Entra ID service principals collected from Microsoft Graph.

**Labels:** `:GraphObject:GraphServicePrincipal`

**Properties:**

- `createdByAppId` - Application ID of the creator
- `agentAppId` - Agent application ID, when present
- `agentIdentityBlueprintId` - Agent identity blueprint ID, when present

- `odataType` - Lowercase `@odata.type`, when returned by Microsoft Graph

- `displayName` - Service principal's display name
- `accountEnabled` - Whether the service principal is enabled
- `alternativeNames` - Alternative names for the service principal
- `appId` - Associated application ID
- `appOwnerOrganizationId` - Owner organization ID
- `publisherName` - Publisher name
- `servicePrincipalType` - Type of service principal
- `loginUrl` - Login URL for the service principal
- `logoutUrl` - Logout URL for the service principal
- `replyUrls` - Array of reply URLs
- `servicePrincipalNames` - Array of service principal names

Agent identity IDs are stored as properties; they do not create separate agent identity relationships.

## Relationships

### Incoming

- **GraphApplication** → [`CREATED`](../edges/created.md) → **GraphServicePrincipal** - Creator identified by `createdByAppId`

### Outgoing

- **GraphServicePrincipal** → [`HAS_PUBLISHED_SCOPE`](../edges/has-published-scope.md) → **GraphApplicationScope** - Scopes from `publishedPermissionScopes`

## Examples

```cypher
MATCH (sp:GraphServicePrincipal)-[:HAS_PUBLISHED_SCOPE]->(scope:GraphApplicationScope)
RETURN sp.displayName, scope.value, scope.adminConsentDisplayName, scope.isEnabled
```
