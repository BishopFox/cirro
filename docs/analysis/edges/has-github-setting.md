# HAS_GITHUB_SETTING

Represents an association between a subnet resource and a GitHub network settings resource.

## Usage

This relationship connects subnet resources to their GitHub network settings:

- **ArmResource** → `HAS_GITHUB_SETTING` → **GitHubNetworkSettings** - Subnet resource linked to a GitHub network setting

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find subnet resources with GitHub network settings
MATCH (sub:ArmResource)-[:HAS_GITHUB_SETTING]->(setting:GitHubNetworkSettings)
RETURN sub.id, setting.businessId
```