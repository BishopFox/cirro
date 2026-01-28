# GraphGroup

Represents Entra ID groups collected from Microsoft Graph.

**Labels:** `:GraphObject:GraphGroup`

**Properties:**

- `id` - Group object ID (primary key)
- `displayName` - Group display name
- `groupTypes` - Array of group types (e.g., ["DynamicMembership", "Unified"])
- `membershipRule` - Membership rule for dynamic groups
- `membershipRuleProcessingState` - State of membership rule processing
- `mailNickname` - Mail nickname for the group
- `onPremisesDomainName` - On-premises domain name
- `onPremisesLastSyncDateTime` - Last synchronization with on-premises
- `onPremisesNetBiosName` - On-premises NetBIOS name
- `onPremisesSamAccountName` - On-premises SAM account name
- `onPremisesSecurityIdentifier` - On-premises security identifier
- `onPremisesSyncEnabled` - Whether on-premises sync is enabled
- `organizationId` - Organization ID
- `securityEnabled` - Whether security is enabled for the group
- `visibility` - Group visibility (Public, Private, etc.)
- `writebackConfigurationEnabled` - Whether writeback is enabled
- `writebackConfigurationGroupType` - Group type for writeback

## Relationships

### Incoming

- **GraphObject** → `MEMBER_OF` → **GraphGroup** - Members of the group
- **GraphObject** → `OWNS` → **GraphGroup** - Owners of the group

### Outgoing

- **GraphGroup** → `MEMBER_OF` → **GraphObject** - Groups this group is a member of

## Examples

```cypher
// Find all security-enabled groups
MATCH (g:GraphGroup)
WHERE g.securityEnabled = true
RETURN g.displayName, g.groupTypes, g.visibility
```

```cypher
// Find dynamic membership groups
MATCH (g:GraphGroup)
WHERE g.membershipRule IS NOT NULL
RETURN g.displayName, g.membershipRule, g.membershipRuleProcessingState
```

```cypher
// Find group owners and members
MATCH (owner:GraphObject)-[:OWNS]->(g:GraphGroup)<-[:MEMBER_OF]-(member:GraphObject)
RETURN g.displayName, collect(DISTINCT owner.displayName) AS owners, collect(DISTINCT member.displayName) AS members
```
