# NSGRule

Represents network security group rules (not an ARM resource itself).

**Labels:** `:NSGRule`

**Properties:**

- `id` - Rule ID (primary key)
- `name` - Rule name
- `type` - Rule type
- `access` - Access type (Allow/Deny)
- `description` - Rule description
- `destinationAddressPrefix` - Destination address prefix
- `destinationAddressPrefixes` - Destination address prefixes (array)
- `destinationPortRange` - Destination port range
- `destinationPortRanges` - Destination port ranges (array)
- `direction` - Traffic direction (Inbound/Outbound)
- `priority` - Rule priority
- `protocol` - Network protocol
- `sourceAddressPrefix` - Source address prefix
- `sourceAddressPrefixes` - Source address prefixes (array)
- `sourcePortRange` - Source port range
- `sourcePortRanges` - Source port ranges (array)

**Relationships:**
- `HAS_RULE` ← NSG
