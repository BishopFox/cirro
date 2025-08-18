# GalleryAppVersion

Represents gallery application versions.

**Labels:** `:ArmResource:GalleryAppVersion`

**Properties:**

- `id` - Gallery application version resource ID (primary key)
- `excludeFromLatest` - Whether excluded from latest
- `installAction` - Install action command
- `removeAction` - Remove action command
- `publishedDate` - Published date
- `packageFileName` - Package file name
- `scriptBehaviorAfterReboot` - Script behavior after reboot
- `source` - Source media link

**Relationships:**
- `HAS_VERSION` ← GalleryApp
- `REFERENCES_PACKAGE` ← VMApplication
