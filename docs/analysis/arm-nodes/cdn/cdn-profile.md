# CdnProfile

Represents Azure Content Delivery Network (CDN) profiles.

**Labels:** `:ArmResource:CdnProfile`

**Properties:**

- `id` - CDN profile resource ID (primary key)
- `frontDoorId` - Front Door ID (if applicable)
- `resourceState` - Current state of the CDN profile
- `sku` - SKU/pricing tier of the CDN profile

**Relationships:**
- `HAS_ENDPOINT` → AfdEndpoint (for Azure Front Door endpoints)