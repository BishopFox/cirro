# Introduction

## Overview

[Cirro](https://github.com/bishopfox/cirro) is a comprehensive security research tool designed to help penetration testers and security researchers map and analyze cloud environments across multiple platforms. While currently focused on Azure cloud environments, Cirro is architected for extensibility to support additional cloud platforms in the future.

For Azure environments, Cirro collects data from both Azure Resource Manager (ARM) APIs and Microsoft Graph APIs, providing a complete picture of your target environment's attack surface.

<div class="grid" markdown>

<div class="card" markdown>
**:material-rocket-launch: [Quick Start](usage/quick-start.md)**

Get up and running with Cirro in minutes. Install, collect, and analyze your first cloud environment (currently Azure).
</div>

</div>

---

!!! warning "Security Notice"

    Cirro is designed for authorized security testing and research. Ensure you have proper permissions before running against any cloud environment. 

## Key Features

=== "Data Collection"

    **Comprehensive Coverage (Azure)**
    
    - :material-microsoft-azure: **Azure Resources**: VMs, storage accounts, Key Vaults, and more
    - :material-account-group: **Identity Data**: Users, groups, applications, service principals
    - :material-shield-account: **RBAC Mappings**: Role assignments and permissions

=== "Authentication"

    **Flexible Authentication (Azure)**
    
    - :material-console: **Azure CLI**: Use existing CLI authentication
    - :material-key-variant: **Access Tokens**: Direct token authentication
    - :material-certificate: **Client Certificates**: Certificate-based auth
    - :material-lock: **Client Secrets**: Service principal authentication

=== "Graph Databases"

    **Database Compatibility**
    
    - :simple-neo4j: **Neo4j**: Industry-standard graph database


## Use Case Examples

<div class="grid" markdown>

<div class="admonition example" markdown>
<p class="admonition-title">Security Testing</p>
Map cloud environments during security assessments to identify:

- Privilege escalation paths
- Misconfigured permissions
- Trust relationships
- Environment reconnaissance
- Attack path planning
</div>

<div class="admonition example" markdown>
<p class="admonition-title">Defensive Security</p>
Strengthen cloud environments by analyzing:

- Security posture assessment
- Access control validation
- Risk identification
- Security monitoring gaps
- Configuration hardening
</div>

</div>