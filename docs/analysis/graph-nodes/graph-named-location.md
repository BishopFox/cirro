# CAPNamedLocation

Represents Conditional Access Policy Named Locations from Microsoft Graph.

**Labels:** `:GraphObject:CAPNamedLocation`

**Properties:**

- `id` - Object ID (primary key)
- `odataType` - OData type
- `createdDateTime` - Creation date/time
- `modifiedDateTime` - Modified date/time
- `deletedDateTime` - Deleted date/time
- `displayName` - Display name
- `isTrusted` - Whether the location is trusted
- `ipRanges` - IP ranges (JSON)
- `countriesAndRegions` - Countries and regions
- `countryLookupMethod` - Country lookup method
- `includeUnknownCountriesAndRegions` - Whether unknown countries/regions are included

## Examples

```cypher
// Find all trusted named locations
MATCH (nl:CAPNamedLocation)
WHERE nl.isTrusted = true
RETURN nl.displayName, nl.ipRanges
```
