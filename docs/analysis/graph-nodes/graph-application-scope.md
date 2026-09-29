# GraphApplicationScope

Represents a permission scope from a service principal's `publishedPermissionScopes` collection.

**Labels:** `:GraphApplicationScope`

**Properties:**

- `id` - Permission scope ID (merge key)
- `adminConsentDescription` - Description shown for administrator consent
- `adminConsentDisplayName` - Display name shown for administrator consent
- `isEnabled` - Whether the scope is enabled
- `type` - Consent type returned by Microsoft Graph
- `userConsentDescription` - Description shown for user consent
- `userConsentDisplayName` - Display name shown for user consent
- `value` - Permission scope value

## Relationships

### Incoming

- **GraphServicePrincipal** → [`HAS_PUBLISHED_SCOPE`](../edges/has-published-scope.md) → **GraphApplicationScope** - Service principal publishing the scope

## Examples

```cypher
MATCH (sp:GraphServicePrincipal)-[:HAS_PUBLISHED_SCOPE]->(scope:GraphApplicationScope)
RETURN sp.displayName, scope.value, scope.type
```
