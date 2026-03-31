# HAS_APPLICATION

Represents the relationship between galleries and their applications.

## Usage

This relationship connects shared image galleries to their gallery applications:

- **Gallery** → `HAS_APPLICATION` → **GalleryApp** - Galleries to their applications

## Properties

No additional properties on the relationship.

## Examples

```cypher
// Find all galleries and their applications
MATCH (gallery:Gallery)-[:HAS_APPLICATION]->(app:GalleryApp)
RETURN gallery.name, app.name, app.supportedOSType
```
