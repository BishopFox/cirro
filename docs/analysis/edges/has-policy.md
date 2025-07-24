# HAS_POLICY

Represents Key Vault access policy relationships.

**Direction:** `(graphObject)-[:HAS_POLICY]->(keyVault)`

**Description:** Indicates that a Graph object (user, service principal, or group) has an access policy defined for a specific Key Vault.

**Common Patterns:**

- Users and service principals have access policies to Key Vaults
- Access policies define permissions for keys, secrets, and certificates
- Multiple principals can have policies for the same Key Vault

**Properties:**
- `certificates` - Array of certificate permissions
- `keys` - Array of key permissions  
- `secrets` - Array of secret permissions

## Query Examples

```cypher
// Find all access policies for Key Vaults
MATCH path = (principal)-[:HAS_POLICY]->(kv:KeyVault)
RETURN path

// Find principals with specific Key Vault permissions
MATCH (principal)-[policy:HAS_POLICY]->(kv:KeyVault)
WHERE 'Get' IN policy.secrets AND 'List' IN policy.secrets
RETURN principal, kv, policy.secrets

// Find principals with Key Vault access
MATCH (principal:GraphObject)-[policy:HAS_POLICY]->(kv:KeyVault)
RETURN principal, COUNT(kv) as keyVaultCount, COLLECT(kv.name) as keyVaults
```
