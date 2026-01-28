# GraphUser

Represents Entra ID users collected from Microsoft Graph.

**Labels:** `:GraphObject:GraphUser`

**Properties:**

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

## Relationships

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
