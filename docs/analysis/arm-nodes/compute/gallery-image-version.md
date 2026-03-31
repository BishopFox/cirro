# GalleryImageVersion

Represents Azure Compute Gallery Image Versions.

**Labels:** `:ArmResource:GalleryImageVersion`

**Properties:**

- `id` - Resource ID (primary key)
- `excludedFromLatest` - Whether excluded from latest
- `publishedDate` - Published date
- `replicaCount` - Replica count
- `storageAccountType` - Storage account type

## Relationships

### Outgoing

- **GalleryImageVersion** → `HAS_SOURCE` → **Disk** - Source OS disk image
- **GalleryImageVersion** → `HAS_SOURCE` → **ComputeImage** - Source compute image

## Examples

```cypher
// Find gallery image versions and their sources
MATCH (giv:GalleryImageVersion)-[:HAS_SOURCE]->(src)
RETURN giv.name, giv.publishedDate, labels(src), src.id
```
