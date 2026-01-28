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

## New Node Types

The following ARM resource types have been recently added:

### Azure Active Directory
- **B2CDirectory** - Entra ID B2C directories

### CDN (Content Delivery Network)
- **CdnProfile** - Azure CDN profiles
- **AfdEndpoint** - Azure Front Door endpoints

### Dashboard & Monitoring
- **GrafanaDashboard** - Azure Managed Grafana instances

### SaaS (Software as a Service)
- **SaasResource** - Azure Marketplace SaaS resources

### Analytics
- **SynapseWorkspace** - Azure Synapse Analytics workspaces

### Development Tools
- **VisualStudioAccount** - Azure DevOps/Visual Studio accounts