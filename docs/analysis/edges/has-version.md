# HAS_VERSION

Represents the relationship between gallery applications and their versions.

## Usage

This relationship connects gallery applications to their version instances:

- **GalleryApp** → `HAS_VERSION` → **GalleryAppVersion** - Gallery applications to their versions

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all gallery applications and their versions
MATCH (app:GalleryApp)-[:HAS_VERSION]->(version:GalleryAppVersion)
RETURN app.name, version.name, version.publishingProfile
```
