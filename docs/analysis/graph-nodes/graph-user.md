# GraphUser

Represents Entra ID users collected from Microsoft Graph.

**Labels:** `:GraphObject:GraphUser`

**Properties:**

- `agentIdentityBlueprintId` - Agent identity blueprint ID, when present
- `identityParentId` - Parent identity ID, when present
- `identityParentType` - Parent identity type, when present

- `odataType` - Lowercase `@odata.type`, when returned by Microsoft Graph

- `id` - User object ID (primary key)
- `accountEnabled` - Whether the user account is enabled
- `city` - User's city
- `companyName` - User's company name
- `country` - User's country
- `creationType` - How the user was created
- `department` - User's department
- `displayName` - User's display name
- `givenName` - User's first name
- `jobTitle` - User's job title
- `lastPasswordChangeDateTime` - When the user's password was last changed or created
- `mail` - User's email address
- `mailNickname` - User's mail nickname
- `mobilePhone` - User's mobile phone number
- `officeLocation` - User's office location
- `onPremisesDomainName` - On-premises domain name
- `onPremisesDistinguishedName` - On-premises distinguished name
- `onPremisesLastSyncDateTime` - Last sync with on-premises
- `onPremisesSyncEnabled` - Whether on-premises sync is enabled
- `onPremisesSamAccountName` - On-premises SAM account name
- `onPremisesSecurityIdentifier` - On-premises security identifier
- `onPremisesUserPrincipalName` - On-premises UPN
- `passwordPolicies` - Password policies applied
- `refreshTokensValidFromDateTime` - When refresh tokens are valid from
- `securityIdentifier` - Security identifier
- `state` - User's state/province
- `surname` - User's last name
- `userPrincipalName` - User principal name (UPN)
- `userType` - Type of user (Member, Guest, etc.)

Agent identity fields remain properties on `GraphUser`; ingestion does not create relationships from `identityParentId` or `agentIdentityBlueprintId`.

Normal user collection retrieves `lastPasswordChangeDateTime` separately and looks up managers for enabled member users. Restricted guest collection does not fetch full user records.

## Relationships

### Incoming

- **GraphUser** ← `MANAGES` ← **GraphObject** - The user's manager. Cirro collects this relationship only for enabled member users (`accountEnabled: true` and `userType: Member`). Users without an assigned or accessible manager have no `MANAGES` relationship.

### Outgoing

- **GraphUser** → `MEMBER_OF` → **GraphObject** - Groups the user is a member of

## Examples

```cypher
// Find all enabled users
MATCH (u:GraphUser)
WHERE u.accountEnabled = true
RETURN u.displayName, u.userPrincipalName, u.jobTitle
```

```cypher
// Find guest users
MATCH (u:GraphUser)
WHERE u.userType = "Guest"
RETURN u.displayName, u.mail, u.companyName
```

```cypher
// Find users and their group memberships
MATCH (u:GraphUser)-[:MEMBER_OF]->(g:GraphGroup)
RETURN u.displayName, collect(g.displayName) AS groups
```

```cypher
// Find enabled member users and their managers
MATCH (manager:GraphObject)-[:MANAGES]->(user:GraphUser)
WHERE user.accountEnabled = true AND user.userType = "Member"
RETURN user.displayName, coalesce(manager.displayName, manager.id) AS manager
```

```cypher
// Review collected password-change timestamps
MATCH (user:GraphUser)
WHERE user.lastPasswordChangeDateTime IS NOT NULL
RETURN user.userPrincipalName, user.lastPasswordChangeDateTime
ORDER BY user.lastPasswordChangeDateTime
```

```cypher
// Find users with agent identity metadata
MATCH (user:GraphUser)
WHERE user.agentIdentityBlueprintId IS NOT NULL OR user.identityParentId IS NOT NULL
RETURN user.displayName, user.odataType, user.agentIdentityBlueprintId,
       user.identityParentId, user.identityParentType
```
