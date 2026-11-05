# GraphServicePrincipal

Represents Entra ID service principals collected from Microsoft Graph.

**Labels:** `:GraphObject:GraphServicePrincipal`

**Properties:**

- `id` - Service principal object ID (primary key)
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

## Relationships

### Incoming

- **GraphApplication** → [`CREATED`](../edges/created.md) → **GraphServicePrincipal** - Creator identified by `createdByAppId`
- **GraphApplication** → [`REPRESENTED_BY`](../edges/represented-by.md) → **GraphServicePrincipal** - Application represented by the service principal, matched by `appId`
- **GraphObject** → [`ASSIGNED_APPROLE`](../edges/assigned-approle.md) → **GraphServicePrincipal** - Principal assigned an app role on this resource
- **GraphObject** → [`OWNS`](../edges/owns.md) → **GraphServicePrincipal** - Owner of the service principal

### Outgoing

- **GraphServicePrincipal** → [`HAS_PUBLISHED_SCOPE`](../edges/has-published-scope.md) → **GraphApplicationScope** - Scopes from `publishedPermissionScopes`
- **GraphServicePrincipal** → [`HAS_APPROLE`](../edges/has-approle.md) → **GraphAppRole** - Roles defined by the service principal

## Examples

```cypher
MATCH (sp:GraphServicePrincipal)-[:HAS_PUBLISHED_SCOPE]->(scope:GraphApplicationScope)
RETURN sp.displayName, scope.value, scope.adminConsentDisplayName, scope.isEnabled
```

```cypher
// Review assignments, including principals with only an object ID
MATCH (principal:GraphObject)-[assignment:ASSIGNED_APPROLE]->(resource:GraphServicePrincipal)
RETURN coalesce(principal.displayName, principal.id) AS principal,
       resource.displayName, assignment.appRoleId
```

```cypher
// Find service principals with agent identity metadata
MATCH (sp:GraphServicePrincipal)
WHERE sp.agentAppId IS NOT NULL OR sp.agentIdentityBlueprintId IS NOT NULL
RETURN sp.displayName, sp.odataType, sp.agentAppId, sp.agentIdentityBlueprintId
```
