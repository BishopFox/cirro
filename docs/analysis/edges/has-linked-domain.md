# HAS_LINKED_DOMAIN

Represents linked domain associations for Azure Communication Services.

**Direction:** `(communicationService)-[:HAS_LINKED_DOMAIN]->(commServiceDomain)`

**Description:** Connects a `CommunicationServices` resource to linked domains referenced in its configuration.

**Properties:** None

## Query Examples

```cypher
// Find communication services and linked domains
MATCH (cs:CommunicationServices)-[:HAS_LINKED_DOMAIN]->(d:CommServiceDomain)
RETURN cs.name, d.id
```
