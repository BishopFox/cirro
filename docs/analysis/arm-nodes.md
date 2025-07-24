# ARM Nodes

This document describes all the Azure Resource Manager (ARM) node types created by Cirro's data ingestion process and their properties.

<h2>Generic ARM Resource</h2>

Base properties inherited by all Azure Resource Manager resources.

**Labels:** `:ArmResource`

**Properties:**

- `id` - Resource ID (primary key, lowercase)
- `kind` - Resource kind
- `location` - Azure region/location
- `name` - Resource name
- `type` - Resource type
- `tags` - Resource tags as key:value pairs

**Relationships:**
- `HAS_IDENTITY` → GraphObject (for managed identities)
