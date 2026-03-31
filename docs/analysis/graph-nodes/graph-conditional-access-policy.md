# ConditionalAccessPolicy

Represents Conditional Access Policies from Microsoft Graph.

**Labels:** `:GraphObject:ConditionalAccessPolicy`

**Properties:**

- `id` - Object ID (primary key)
- `createdDateTime` - Creation date/time
- `modifiedDateTime` - Modified date/time
- `displayName` - Display name
- `state` - Policy state (enabled/disabled/enabledForReportingButNotEnforced)
- `templateId` - Template ID
- `conditions` - Conditions (JSON)
- `grantControls` - Grant controls (JSON)

## Examples

```cypher
// Find all enabled conditional access policies
MATCH (cap:ConditionalAccessPolicy)
WHERE cap.state = 'enabled'
RETURN cap.displayName, cap.conditions
```
