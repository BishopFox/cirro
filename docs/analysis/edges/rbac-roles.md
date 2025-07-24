# Dynamic Role Relationships

RBAC relationships are created dynamically based on Azure role assignments. The relationship type matches the role name.

**Direction:** `(principal)-[:ROLE_NAME]->(scope)`

**Description:** Indicates that a security principal (user, group, service principal) has been assigned a specific role at a particular scope.

**Common Role Types:**

- `Owner` - Full access to all resources
- `Contributor` - Can create and manage all types of Azure resources
- `Reader` - Can view existing Azure resources
- `User Access Administrator` - Can manage user access to Azure resources
- Various service-specific roles (e.g., `Storage Account Contributor`, `Virtual Machine Contributor`)

**Properties:**

- `id` - Role assignment ID
- `description` - Role assignment description
- `roleName` - Name of the assigned role
- `roleType` - Type of role (BuiltInRole, CustomRole)
- `actions` - Array of allowed actions
- `notActions` - Array of denied actions
- `dataActions` - Array of allowed data actions
- `notDataActions` - Array of denied data actions

## Query Examples

```cypher
// Find all users with Owner roles
MATCH path=(u:GraphUser)-[:Owner]->(r)
RETURN path

// Find all Contributors at subscription level
MATCH path=(principal)-[:Contributor]->(sub:Subscription)
RETURN path

// Find service principals with elevated permissions
MATCH path=(sp:GraphServicePrincipal)-[role:Owner|Contributor]->(scope)
RETURN path

// Find all role assignments for a specific user
MATCH path=(u:GraphUser {userPrincipalName: 'admin@company.com'})-[role]->(scope)
WHERE type(role) IN ['Owner', 'Contributor', 'Reader', 'User Access Administrator']
RETURN path
```
