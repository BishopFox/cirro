# GalleryApp

Represents gallery applications within Azure Compute Galleries.

**Labels:** `:ArmResource:GalleryApp`

**Properties:**

- `id` - Gallery application resource ID (primary key)
- `supportedOSType` - Supported operating system type

**Relationships:**
- `HAS_APPLICATION` ← Gallery
- `HAS_VERSION` → GalleryAppVersion
