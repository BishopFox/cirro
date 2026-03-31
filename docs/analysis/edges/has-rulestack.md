# HAS_RULESTACK

Represents Palo Alto Cloud NGFW firewall association to its local rules stack.

**Direction:** `(firewall)-[:HAS_RULESTACK]->(localRulestack)`

**Description:** Links `PanCloudNgfwFirewall` resources to their associated `PanCloudNgfwLocalRulestack`.

**Properties:** None

## Query Examples

```cypher
// Find firewalls and associated rule stacks
MATCH (fw:PanCloudNgfwFirewall)-[:HAS_RULESTACK]->(rs:PanCloudNgfwLocalRulestack)
RETURN fw.name, rs.id
```
