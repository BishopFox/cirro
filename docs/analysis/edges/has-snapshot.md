# HAS_SNAPSHOT

Represents the relationship between storage resources and their snapshots.

## Usage

This relationship connects storage resources to their snapshots:

- **ArmResource** → `HAS_SNAPSHOT` → **Snapshot** - Storage resources to their snapshots

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all resources with snapshots
MATCH (resource:ArmResource)-[:HAS_SNAPSHOT]->(snapshot:Snapshot)
RETURN resource.name, snapshot.name, snapshot.diskSizeGB
```
