# VMApplication

Represents virtual machine applications.

**Labels:** `:ArmResource:VMApplication`

**Properties:**

- `id` - VM application resource ID (primary key)
- `enableAutomaticUpgrade` - Whether automatic upgrade is enabled
- `manuallyManaged` - Whether manually managed
- `treatFailureAsDeploymentFailure` - Whether to treat failure as deployment failure

**Relationships:**
- `REFERENCES_PACKAGE` → GalleryAppVersion
- `HAS_VMAPP` ← VirtualMachine
