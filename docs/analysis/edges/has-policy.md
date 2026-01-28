# HAS_POLICY

Represents policy relationships in Entra ID and Azure resources.

**Direction:** 
- `(graphObject)-[:HAS_POLICY]->(keyVault)` - Key Vault access policies
- `(org:GraphOrg)-[:HAS_POLICY]->(policy:GraphPolicy)` - Organization policies

**Description:** This relationship has two main contexts:

1. **Key Vault Access Policies**: Indicates that a Graph object (user, service principal, or group) has an access policy defined for a specific Key Vault.
2. **Organization Policies**: Links Entra ID organizations to their configured policies that define organizational settings, user permissions, and security rules.

## Key Vault Access Policies

**Common Patterns:**
- Users and service principals have access policies to Key Vaults
- Access policies define permissions for keys, secrets, and certificates
- Multiple principals can have policies for the same Key Vault

**Properties:**
- `certificates` - Array of certificate permissions
- `keys` - Array of key permissions  
- `secrets` - Array of secret permissions

## Organization Policies

**Common Patterns:**
- Organizations have authorization policies that govern user behavior
- Policies define default permissions and restrictions
- Each organization typically has one policy per policy type

**Properties:** None (properties are stored on the GraphPolicy node)

## Query Examples

```cypher
// Find all Key Vault access policies
MATCH path = (principal)-[:HAS_POLICY]->(kv:KeyVault)
RETURN path

// Find principals with specific Key Vault permissions
MATCH (principal)-[policy:HAS_POLICY]->(kv:KeyVault)
WHERE 'Get' IN policy.secrets AND 'List' IN policy.secrets
RETURN principal, kv, policy.secrets

// Find all organization policies
MATCH (org:GraphOrg)-[:HAS_POLICY]->(policy:GraphPolicy)
RETURN org.displayName, policy.type, policy.displayName

// Find organizations that allow user consent for risky apps
MATCH (org:GraphOrg)-[:HAS_POLICY]->(policy:GraphPolicy)
WHERE policy.allowUserConsentForRiskyApps = true
RETURN org.displayName, policy.displayName

// Find all authorization policies across all organizations
MATCH (org:GraphOrg)-[:HAS_POLICY]->(policy:GraphPolicy)
WHERE policy.type = 'authorizationpolicy'
RETURN org.displayName, policy.allowEmailVerifiedUsersToJoinOrganization, policy.blockMsolPowerShell
```
