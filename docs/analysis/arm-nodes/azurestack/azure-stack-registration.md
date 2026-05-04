# AzureStackRegistration

Represents Azure Stack registration resources, which link on-premises Azure Stack hardware to an Azure subscription.

**Labels:** `:ArmResource:AzureStackRegistration`

**Properties:**

- `id` - Registration resource ID (primary key)
- `cloudId` - Cloud identifier for the registered Azure Stack

## Relationships

### Outgoing

- **AzureStackRegistration** → `REGISTERED_BY` → **GraphObject** - Identity that registered this Azure Stack

## Examples

```cypher
// Find all Azure Stack registrations
MATCH (r:AzureStackRegistration)
RETURN r.id, r.cloudId
```

```cypher
// Find registrations and who registered them
MATCH (r:AzureStackRegistration)-[:REGISTERED_BY]->(obj:GraphObject)
RETURN r.id, r.cloudId, obj.id
```
