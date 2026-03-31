# REFERENCES_PACKAGE

Represents the relationship between VM applications and their referenced packages.

## Usage

This relationship connects VM applications to the packages they reference:

- **VMApplication** → `REFERENCES_PACKAGE` → **ArmResource** - VM applications to their package references

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all VM applications and their package references
MATCH (vmapp:VMApplication)-[:REFERENCES_PACKAGE]->(pkg:ArmResource)
RETURN vmapp.name, pkg.name, pkg.type
```
