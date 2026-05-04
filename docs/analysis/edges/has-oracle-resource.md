# HAS_ORACLE_RESOURCE

Represents the relationship between an ARM resource and Oracle database resources hosted within it.

**Direction:** `(subnet)-[:HAS_ORACLE_RESOURCE]->(oracleDb)`

**Description:** Indicates that an ARM resource (typically a subnet) contains or is associated with an Oracle Autonomous Database resource.

**Properties:** None

## Usage

- **ArmResource** → `HAS_ORACLE_RESOURCE` → **OracleAutonomousDB** - Subnet or parent resource associated with an Oracle Autonomous Database

## Query Examples

```cypher
// Find all Oracle databases and their parent resources
MATCH (src:ArmResource)-[:HAS_ORACLE_RESOURCE]->(db:OracleAutonomousDB)
RETURN src.id, db.displayName, db.dbVersion

// Find Oracle databases accessible via private endpoint
MATCH (src:ArmResource)-[:HAS_ORACLE_RESOURCE]->(db:OracleAutonomousDB)
WHERE db.privateEndpointIp IS NOT NULL
RETURN src.id, db.displayName, db.privateEndpointIp
```
