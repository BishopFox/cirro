# GitHub Network Settings

Represents Azure GitHub network settings resources used to associate GitHub connectivity settings with a subnet.

**Labels:** `:ArmResource:GithubNetworkSettings`

**Properties:**

- `id` - Resource ID (primary key)
- `businessId` - GitHub business identifier associated with the network setting

## Relationships

### Incoming

- **ArmResource** → `HAS_GITHUB_SETTING` → **GithubNetworkSettings** - Subnet resource associated with this GitHub network setting

## Examples

```cypher
// Find GitHub network settings and their linked subnet resources
MATCH (sub:ArmResource)-[:HAS_GITHUB_SETTING]->(setting:GithubNetworkSettings)
RETURN sub.id, setting.businessId
```