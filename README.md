<p align='center'><img src='docs/logo.png' alt='logo' height="400"/><br>
</p>

Cirro is an extensible security research platform that enables researchers and penetration testers to collect, analyze, and visualize cloud environments and identity relationships through graph databases. Built with a modular architecture, Cirro can be extended to support multiple platforms and data sources.

You can check out the [Documentation](https://crispy-adventure-qre5p1k.pages.github.io/) for more info.

## Features

- **Multi-platform Data Collection**: Extensible architecture supporting multiple cloud platforms and identity providers
- **Flexible Authentication**: Support for various authentication methods depending on the target platform
- **Cross-platform**: Available for Windows, macOS, and Linux
- **Modular Design**: Optional platform functionality through feature flags and extensible plugin architecture
- **Network Topology Analysis**: Support for network infrastructure platforms like Tailscale

## Architecture

Cirro consists of two main components:

- **`cirro`**: The primary data collection tool that gathers information from various platforms and APIs
- **`cirro-graph`**: The ingestion tool that loads collected data into graph databases (Neo4j or Memgraph)

The modular architecture allows for easy extension to support additional platforms beyond the currently supported ones.

## CLI Structure

Cirro uses a hierarchical command structure organized by platform:

```
cirro <platform> <action> <auth-method> [options]
```

### Supported Platforms

**Azure (az)**
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

**Tailscale (ts)**
Network topology analysis for Tailscale environments:

```bash
# Tailscale data collection
cirro ts collect <auth-method> [options]
```

*Additional platforms can be added through the extensible plugin architecture.*

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

By default, Cirro includes all available platform support. To build with specific platform features:

```bash
# Build minimal version without platform-specific features
cargo build --release --no-default-features

# Build with specific platform support
cargo build --release --features azure
cargo build --release --features tailscale
```

## Data Ingestion

Neo4j and Memgraph are supported as graph databases. There are docker-compose files in the [tools](/tools/) directory to assist with containerized databases. 

After collecting data, ingest it into your graph database:

```bash
# Ingest data for specific platforms
cirro-graph ingest --file cirro_output.db --type az     # Azure data
cirro-graph ingest --file cirro_output.db --type ts     # Tailscale data
```

## Dashboard

CirroDash can be located here: [https://github.com/bishopfox/cirrodash](CirroDash)

## Debug Mode

Enable debug logging for detailed information:

```bash
# Collection debug mode
cirro az collect azcli --debug
cirro ts collect api-key --debug

# Ingestion debug mode
cirro-graph ingest --file cirro_output.db --type az --debug
```

---

**Note**: Cirro is designed for authorized security testing and research. Ensure you have proper permissions before running against any cloud or network environment.
