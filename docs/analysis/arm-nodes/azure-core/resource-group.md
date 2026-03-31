# ResourceGroup

Represents Azure resource groups - logical containers for Azure resources.

**Labels:** `:ResourceGroup:ArmResource`

**Properties:**

- `id` - Resource group ID (primary key)
- `name` - Resource group name
- `location` - Azure region
- `type` - Resource type
- `subscriptionId` - Parent subscription ID

**Relationships:**
- `CONTAINS` → Various ARM resources
