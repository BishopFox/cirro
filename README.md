<p align='center'><img src='docs/logo.png' alt='logo' height="400"/><br>
</p>

Cirro is a tool that enables security researchers and penetration testers to collect, analyze, and visualize Azure cloud environments and Entra ID relationships through graph databases.

You can check out the [Documentation](https://crispy-adventure-qre5p1k.pages.github.io/) for more info.

## Features

- **Multi-source Data Collection**: Gather data from both Azure Resource Manager (ARM) APIs and Microsoft Graph APIs
- **Flexible Authentication**: Support for multiple authentication methods including Azure CLI, access tokens, client secrets, and client certificates
- **Graph Database Support**: Compatible with both **Neo4j** and **Memgraph** graph databases
- **Multi-cloud Support**: Works with Azure Public, China, Germany, and US Government clouds
- **Cross-platform**: Available for Windows, macOS, and Linux
- **Modular Design**: Optional Azure functionality through feature flags

## Architecture

Cirro consists of two main components:

- **`cirro`**: The primary data collection tool that gathers information from Azure and Microsoft Graph APIs
- **`cirro-graph`**: The ingestion tool that loads collected data into graph databases (Neo4j or Memgraph)

## CLI Structure

Cirro uses a hierarchical command structure organized by platform:

```
cirro <platform> <action> <auth-method> [options]
```

### Azure Commands

All Azure-related functionality is accessed through the `az` subcommand:

```bash
# Data collection
cirro az collect <auth-method> [options]

# Available authentication methods:
cirro az collect azcli           # Azure CLI authentication
cirro az collect client-secret   # Client ID and secret
cirro az collect client-cert     # Client certificate
cirro az collect access-token    # Pre-obtained access token
```

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [releases page](https://github.com/bishopfox/cirro/releases).

### Building from Source

```bash
git clone https://github.com/bishopfox/cirro.git
cd cirro
cargo build --release
```

Binaries will be available in `target/release/`.

#### Build Options

By default, Cirro includes all functionality. To build with specific platform support:

```bash
# Build without Azure features
cargo build --release --no-default-features

# Build with specific features
cargo build --release --features azure
```

## Data Ingestion

Neo4j is supported as the graph database. There are two docker-compose files in the [tools](/tools/) directory to assist with containerized databases. 

After collecting data, ingest it into your graph database:

```bash
cirro-graph --file cirro_output.db
```

## Dashboard

CirroDash can be located here: [https://github.com/bishopfox/cirrodash](CirroDash)

## Debug Mode

Enable debug logging for detailed information:

```bash
cirro az collect azcli --debug
cirro-graph --file cirro_output.db --debug
```

---

**Note**: Cirro is designed for authorized security testing and research. Ensure you have proper permissions before running against any Azure environment.
