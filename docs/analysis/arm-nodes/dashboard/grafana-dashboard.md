# GrafanaDashboard

Represents Azure Managed Grafana instances.

**Labels:** `:ArmResource:GrafanaDashboard`

**Properties:**

- `id` - Grafana instance resource ID (primary key)
- `apiKey` - API key for the Grafana instance
- `creatorCanAdmin` - Whether the creator has admin privileges
- `deterministicOutboundIp` - Whether deterministic outbound IP is enabled
- `endpoint` - Grafana endpoint URL
- `smtpEnabled` - Whether SMTP is enabled
- `grafanaMajorVersion` - Major version of Grafana
- `grafanaVersion` - Full version of Grafana
- `publicNetworkAccess` - Public network access setting