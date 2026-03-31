# NSG (Network Security Group)

Represents network security groups and their rules.

**Labels:** `:ArmResource:NSG`

**Properties:**

- `id` - NSG resource ID (primary key)

**Relationships:**
- `HAS_RULE` → NSGRule
- `HAS_NSG` ← NetworkInterface
