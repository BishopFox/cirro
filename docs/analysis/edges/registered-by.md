# REGISTERED_BY

Represents the relationship between an Azure Stack resource and the identity that registered it.

**Direction:** `(resource)-[:REGISTERED_BY]->(graphObject)`

**Description:** Indicates that an Azure Stack resource (registration or edge machine) was registered by a specific identity (Graph object).

**Properties:** None

## Usage

- **AzureStackRegistration** → `REGISTERED_BY` → **GraphObject** - Azure Stack registration associated with its registering identity
- **AzureStackEdgeMachine** → `REGISTERED_BY` → **GraphObject** - Edge machine associated with its registering identity

## Query Examples

```cypher
// Find all Azure Stack resources and their registering identities
MATCH (r:ArmResource)-[:REGISTERED_BY]->(obj:GraphObject)
WHERE r:AzureStackRegistration OR r:AzureStackEdgeMachine
RETURN r.id, labels(r), obj.id
```
