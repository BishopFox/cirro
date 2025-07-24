# Introduction

## Overview

[Cirro](https://github.com/bishopfox/cirro) is a comprehensive security research tool that helps penetration testers and security researchers map and analyze Azure cloud environments. By collecting data from both Azure Resource Manager (ARM) APIs and Microsoft Graph APIs, Cirro provides a complete picture of your target environment's attack surface.

<div class="grid" markdown>

<div class="card" markdown>
**:material-rocket-launch: [Quick Start](quick-start.md)**

Get up and running with Cirro in minutes. Install, collect, and analyze your first Azure environment.
</div>

<div class="card" markdown>
**:material-feature-search: [Key Features](features.md)**

Explore Cirro's powerful data collection, authentication, and graph database capabilities.
</div>

<div class="card" markdown>
**:material-floor-plan: [Architecture](architecture.md)**

Understand how Cirro's two-stage architecture works and how data flows through the system.
</div>

<div class="card" markdown>
**:material-account-group: [Use Cases](use-cases.md)**

Discover how security professionals use Cirro for penetration testing, research, and compliance.
</div>

</div>

---

!!! warning "Security Notice"

    Cirro is designed for authorized security testing and research. Ensure you have proper permissions before running against any Azure environment. 

## Key Features

=== "Data Collection"

    **Comprehensive Coverage**
    
    - :material-microsoft-azure: **Azure Resources**: VMs, storage accounts, Key Vaults, and more
    - :material-account-group: **Identity Data**: Users, groups, applications, service principals
    - :material-shield-account: **RBAC Mappings**: Role assignments and permissions

=== "Authentication"

    **Flexible Authentication**
    
    - :material-console: **Azure CLI**: Use existing CLI authentication
    - :material-key-variant: **Access Tokens**: Direct token authentication
    - :material-certificate: **Client Certificates**: Certificate-based auth
    - :material-lock: **Client Secrets**: Service principal authentication

=== "Graph Databases"

    **Database Compatibility**
    
    - :simple-neo4j: **Neo4j**: Industry-standard graph database
    - :material-graph: **Memgraph**: High-performance in-memory graph database

## Use Case Examples

<div class="grid" markdown>

<div class="admonition example" markdown>
<p class="admonition-title">Security Testing</p>
Map Azure environments during security assessments to identify:

- Privilege escalation paths
- Misconfigured permissions
- Trust relationships
- Environment reconnaissance
- Attack path planning
</div>

<div class="admonition example" markdown>
<p class="admonition-title">Defensive Security</p>
Strengthen Azure environments by analyzing:

- Security posture assessment
- Access control validation
- Risk identification
- Security monitoring gaps
- Configuration hardening
</div>

</div>