# HAS_PARAMETER

Represents the relationship between runbooks and their parameters.

## Usage

This relationship connects automation runbooks to their defined parameters:

- **Runbook** → `HAS_PARAMETER` → **RunbookParameter** - Runbooks to their parameters

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all runbooks and their parameters
MATCH (rb:Runbook)-[:HAS_PARAMETER]->(param:RunbookParameter)
RETURN rb.name, param.name, param.type, param.isMandatory
```
