# Frequently Asked Questions (FAQ)

## What is Cirro?

Cirro is a comprehensive security research tool designed for penetration testers and security researchers to map and analyze Azure cloud environments. It provides a complete view of your target environment's attack surface by collecting data from both Azure Resource Manager (ARM) APIs and Microsoft Graph APIs.

## What permissions does Cirro need?

For comprehensive data collection, Cirro requires:

**Azure Resource Manager (ARM)**:

- `Reader` role on subscriptions/resource groups you want to enumerate
- Additional permissions for specific resource types (e.g., Key Vault access)

**Microsoft Graph**:

- `User.Read.All` - Read user profiles
- `Group.Read.All` - Read group information
- `Application.Read.All` - Read application registrations
- `Directory.Read.All` - Read directory data
- `RoleManagement.Read.All` - Read role assignments

!!! tip "Enrichment Permissions"
    Permissions required for enrichment features dependings on the resource being enumerated.

## What visualization tools can I use with Cirro?

Since Cirro loads data into standard graph databases, you can use any compatible visualization tool:

**For Neo4j**:
- Neo4j Browser (built-in)
- Neo4j Bloom
- NeoDash (included configuration)

**For Memgraph**:
- Memgraph Lab
- NeoDash

## How is Cirro different from AzureHound?

While both tools help with Azure security assessment, they take fundamentally different approaches:

### Collection Scope: Management Plane vs Data Plane

In Azure, there are two primary planes of operation that security tools can analyze. 

The **management plane** handles the configuration, deployment, and administration of Azure resources. This includes APIs like Azure Resource Manager (ARM) and Microsoft Graph that control *how* resources are configured and *who* has access to them. 

The **data plane** deals with the actual data and operations within those resources, such as reading files from storage accounts, querying databases, or accessing secrets from Key Vaults. Understanding this distinction is crucial when choosing security assessment tools, as each plane reveals different aspects of the attack surface and potential security risks.

**Cirro** focuses on the **management and data planes**:

- Collects comprehensive Azure Resource Manager (ARM) data
- Enumerates **all** Azure resources (VMs, storage accounts, Key Vaults, etc.)
- Maps infrastructure relationships and dependencies
- Provides visibility into resource configurations and security settings
- Analyzes both identity (Microsoft Graph) and infrastructure (ARM) in a unified view

**AzureHound** focuses on the **management plane with minimal data plane analysis**:

- Primarily collects Microsoft Graph/Entra ID identity data
- Enumerates users, groups, applications, and service principals
- Maps identity relationships to limited resource types
- Limited visibility into actual Azure infrastructure configurations

### When should I use Cirro vs AzureHound?

<div class="grid cards" markdown>

-   :material-cloud-outline:{ .lg .middle } __Use Cirro when:__

    ---

    - You need comprehensive Azure infrastructure visibility
    - Your engagement includes Azure resources beyond identity
    - You want to understand resource relationships and dependencies
    - You need to analyze storage account security configurations
    - You're looking for infrastructure-level misconfigurations
    - You want a unified view of identity and infrastructure

-   :material-account-supervisor:{ .lg .middle } __Use AzureHound when:__

    ---

    - You're performing traditional Active Directory-style assessments where you are already using BloodHound in hybrid environments
    - You are using the *Hound toolkit to keep a unified picture of multiple environments
    - You don't need a comprehensive view of Azure resources but want to focus only on resources with a high rate of success for privilege escalation and lateral movement  

</div>

<h4>Use both when:</h4>
- You want complete coverage of both identity and infrastructure
- Your assessment scope includes both Azure AD and Azure resources
- You need to correlate identity privileges with resource access
- You're performing comprehensive cloud security assessments

## How is Cirro different than RoadRecon?

RoadRecon and Cirro both target Azure environments but serve different phases of security assessment and have distinct architectural approaches. **RoadRecon** is primarily designed for reconnaissance, focusing on gathering Entra ID information through web scraping and REST API enumeration. It excels at collecting identity data when you have limited credentials or need to perform reconnaissance without extensive permissions. RoadRecon stores its data in a custom SQLite database with a web-based frontend for analysis, making it ideal for quick reconnaissance and initial environment mapping.

**Cirro**, in contrast, is designed for comprehensive security analysis once you have established access to an Azure environment. While RoadRecon focuses heavily on Entra ID reconnaissance, Cirro provides deep visibility into both the identity layer (Microsoft Graph) and the entire Azure infrastructure layer (ARM resources). Where RoadRecon might help you discover what users and applications exist in a tenant, Cirro helps you understand how those identities relate to actual Azure resources, their configurations, permissions, and potential attack paths across the entire cloud infrastructure.
