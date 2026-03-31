# AfdEndpoint

Represents Azure Front Door (AFD) endpoints within CDN profiles.

**Labels:** `:ArmResource:AfdEndpoint`

**Properties:**

- `id` - AFD endpoint resource ID (primary key)
- `deploymentStatus` - Current deployment status of the endpoint
- `hostName` - Host name of the endpoint
- `enabledState` - Whether the endpoint is enabled or disabled
- `enforceMtls` - Whether mutual TLS is enforced

**Relationships:**
- Connected to CDN Profile via `HAS_ENDPOINT` relationship (reverse direction)