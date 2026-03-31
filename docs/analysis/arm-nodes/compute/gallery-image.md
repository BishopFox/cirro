# GalleryImage

Represents Azure Compute Gallery Image definitions.

**Labels:** `:ArmResource:GalleryImage`

**Properties:**

- `id` - Resource ID (primary key)
- `architecture` - CPU architecture (e.g. x64, Arm64)
- `osType` - OS type (Windows or Linux)
- `hyperVGeneration` - Hyper-V generation (V1 or V2)
- `offer` - Image offer identifier
- `publisher` - Image publisher
- `sku` - Image SKU
- `osState` - OS state (Generalized or Specialized)
- `provisioningState` - Provisioning state

## Relationships

### Incoming

- **Gallery** → `HAS_GALLERY_IMAGE` → **GalleryImage** - Parent gallery contains this image definition

### Outgoing

- **GalleryImage** → `HAS_VERSION` → **GalleryImageVersion** - Versions of this image definition

## Examples

```cypher
// Find gallery images and their parent galleries
MATCH (g)-[:HAS_GALLERY_IMAGE]->(gi:GalleryImage)
RETURN g.name, gi.name, gi.osType, gi.publisher, gi.offer, gi.sku
```
