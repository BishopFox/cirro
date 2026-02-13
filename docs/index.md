# Introduction

## Overview

[Cirro](https://github.com/bishopfox/cirro) is a comprehensive security research tool designed to help penetration testers and security researchers map and analyze cloud environments across multiple platforms. Today, the project includes modules for Azure and Tailscale, with an architecture designed to support additional platforms over time.

Cirro uses a modular workflow with platform-specific collectors and graph schema mappings. This allows the same core tooling to ingest different data models and relationship structures without changing analyst workflows.

<div class="grid" markdown>

<div class="card" markdown>
**:material-rocket-launch: [Quick Start](usage/quick-start.md)**

Get up and running with Cirro in minutes. Install, collect data, ingest into Neo4j, and begin graph analysis.
</div>

</div>

---

!!! warning "Security Notice"

    Cirro is designed for authorized security testing and research. Ensure you have proper permissions before running against any cloud environment. 

## Key Features

=== "Data Collection"

    **Platform-Aware Collection**
    
    - :material-cloud-outline: **Multi-source Inputs**: Collect from supported cloud, identity, and network APIs
    - :material-account-group: **Relationship Data**: Capture principals, resources, memberships, and trust links
    - :material-puzzle: **Extensible Modules**: Add new collectors through feature-based platform modules

=== "Authentication"

    **Flexible Authentication**
    
    - :material-console: **CLI-based Auth**: Reuse authenticated local tooling where available
    - :material-key-variant: **Token-based Auth**: Use pre-obtained access tokens
    - :material-certificate: **Certificate Auth**: Authenticate with client certificates
    - :material-lock: **Secret-based Auth**: Authenticate with service credentials

=== "Graph Databases"

    **Graph Workflow**
    
    - :simple-neo4j: **Neo4j Ingestion**: Load collected data into a graph database for analysis
    - :material-graph-outline: **Schema Mapping**: Transform platform data into consistent nodes and edges
    - :material-export-variant: **Export Support**: Export graph data for downstream tooling

=== "Schema Extensibility"

    **YAML-Driven Mapping Extensibility**

    - :material-code-braces: **Platform Logic in Code**: New platform collectors and ingestors are implemented as Rust modules
    - :material-file-document-edit: **YAML Graph Specs**: Extend node/edge mappings through YAML definitions used by ingestion
    - :material-source-branch: **Reusable Pipeline**: Keep a consistent ingest and analysis workflow across supported sources


## Use Case Examples

<div class="grid" markdown>

<div class="admonition example" markdown>
<p class="admonition-title">Security Testing</p>
Map complex environments during security assessments to identify:

- Privilege escalation paths
- Misconfigured permissions
- Trust relationships
- Environment reconnaissance
- Attack path planning
</div>

<div class="admonition example" markdown>
<p class="admonition-title">Defensive Security</p>
Strengthen platform security posture by analyzing:

- Security posture assessment
- Access control validation
- Risk identification
- Security monitoring gaps
- Configuration hardening
</div>

</div>