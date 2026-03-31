# ServerFarm

Represents Azure App Service plans.

**Labels:** `:ArmResource:ServerFarm`

**Properties:**

- `id` - Server farm resource ID (primary key)
- `skuName` - SKU name
- `skuTier` - SKU tier
- `skuSize` - SKU size
- `skuFamily` - SKU family
- `skuCapacity` - SKU capacity

**Relationships:**
- `HOSTS_SITE` → WebSite
