# HAS_RULE

Represents network security group rule ownership relationships.

**Direction:** `(nsg)-[:HAS_RULE]->(nsgRule)`

**Description:** Indicates that a Network Security Group contains a specific security rule for traffic filtering.

**Common Patterns:**
- NSGs contain multiple security rules (both default and custom)
- Rules define allow/deny traffic policies
- Rules have priorities that determine evaluation order

**Properties:** None

## Query Examples

```cypher
// Find all rules in network security groups
MATCH path = (nsg:NSG)-[:HAS_RULE]->(rule:NSGRule)
RETURN path

// Find NSGs with many rules
MATCH (nsg:NSG)-[:HAS_RULE]->(rule:NSGRule)
WITH nsg, COUNT(rule) as ruleCount
WHERE ruleCount > 10
RETURN nsg, ruleCount
ORDER BY ruleCount DESC

// Find allow rules by priority
MATCH (nsg:NSG)-[:HAS_RULE]->(rule:NSGRule)
WHERE rule.access = 'Allow'
RETURN nsg, rule
ORDER BY rule.priority

// Find permissive rules (allow any source to any destination)
MATCH (nsg:NSG)-[:HAS_RULE]->(rule:NSGRule)
WHERE rule.access = 'Allow' 
  AND rule.sourceAddressPrefix = '*' 
  AND rule.destinationAddressPrefix = '*'
RETURN nsg, rule

// Find rules allowing specific ports
MATCH (nsg:NSG)-[:HAS_RULE]->(rule:NSGRule)
WHERE rule.destinationPortRange CONTAINS '22' OR rule.destinationPortRange CONTAINS '3389'
RETURN nsg, rule, rule.destinationPortRange

// Find network security topology with rules
MATCH path = (vm:VirtualMachine)-[:HAS_NIC]->(nic:NetworkInterface)-[:HAS_NSG]->(nsg:NSG)-[:HAS_RULE]->(rule:NSGRule)
RETURN path
```
