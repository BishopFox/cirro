<p align='center'><img src='docs/logo.png' alt='logo' height="400"/><br>
</p>

Cirro is an extensible security research platform that enables researchers and penetration testers to collect, analyze, and visualize cloud environments and identity relationships through graph databases. Built with a modular architecture, Cirro can be extended to support multiple platforms and data sources.

You can check out the [Documentation](https://bishopfox.github.io/cirro) for more info.

## Features

- **Multi-platform Data Collection**: Extensible architecture supporting multiple cloud platforms and identity providers
- **Flexible Authentication**: Support for various authentication methods depending on the target platform
- **Cross-platform**: Available for Windows, macOS, and Linux
- **Modular Design**: Optional platform functionality through feature flags and extensible plugin architecture
- **Network Topology Analysis**: Support for network infrastructure platforms like Tailscale

## Architecture

Cirro has two main functional areas:

- **Collection (`cirro collect`)**: Gathers information from various platforms and APIs
- **Graph Operations (`cirro graph`)**: Manages graph database operations including data ingestion and export

The modular architecture uses feature flags to enable platform-specific functionality, allowing users to build only the components they need.

## CLI Structure

Cirro uses a hierarchical command structure organized by function and platform:

```
cirro <function> <platform> <command> [options]
```

### Data Collection

**Azure (collect az)**

```bash
# Available authentication methods:
cirro collect az azcli           # Azure CLI authentication
cirro collect az client-secret   # Client ID and secret
cirro collect az client-cert     # Client certificate
cirro collect az access-token    # Pre-obtained access token
cirro collect az user-pass       # Username and password
```

**Tailscale (collect ts)**

```bash
# Tailscale data collection
cirro collect ts <auth-method> [options]
```

### Graph Operations

**Data Ingestion (graph ingest)**

```bash
# Ingest collected data into graph database
cirro graph ingest --type <platform> --file <data-file> [database options]
```

**Data Export (graph export)**

```bash
# Export graph data to various formats
cirro graph export --format <format> [options]
```

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [releases page](https://github.com/bishopfox/cirro/releases). The releases are built with all features enabled.

### Building from Source

```bash
git clone https://github.com/bishopfox/cirro.git
cd cirro
cargo build --release
```

**NOTE: BUILDING WITH --RELEASE IS IMPORTANT FOR GRAPH FUNCTIONALITY SINCE IT EMBEDS YAML CONFIGURATION FILES IN THE BINARY!**

The binary will be available at `target/release/cirro`.

#### Build Options

By default, Cirro includes all available functionality. To build with specific features:

```bash
# Build with only collection features
cargo build --release --no-default-features --features collector

# Build with only graph features
cargo build --release --no-default-features --features graph

# Build with specific platform support
cargo build --release --no-default-features --features "azure"
cargo build --release --no-default-features --features "tailscale"
```

## Data Ingestion

Cirro uses Neo4j as the backend database. There are docker-compose files in the [tools](/tools/) directory to assist with containerized databases. 

After collecting data, ingest it into your graph database:

```bash
# Ingest data for specific platforms
cirro graph ingest --type az --file cirro_output.db         # Azure data
cirro graph ingest --type ts --file cirro_ts_socket.json    # Tailscale data

# Specify custom database connection
cirro graph ingest --type az --file cirro_output.db \
  --server bolt://localhost:7687 \
  --user neo4j \
  --password password
```

## Dry-Run Mode

Preview what would be ingested and post-processed without writing any data to the graph database:

```bash
# Preview Azure ingestion and see which resource types have no implemented specs
cirro graph ingest --type az --file cirro_output.db --dry-run

# Preview Tailscale ingestion
cirro graph ingest --type ts --file cirro_ts_socket.json --dry-run
```

In dry-run mode, Cirro will:

- Report each spec and its row count that **would** be processed
- Skip all Neo4j writes (no ingestion or post-processing queries execute)
- For Azure, list resource types in the input data that have no implemented spec

## Dashboard

CirroDash can be located here: [https://github.com/bishopfox/cirrodash](https://github.com/bishopfox/cirrodash)

## Debug Mode

Enable debug logging for detailed information:

```bash
# Collection debug mode
cirro collect az azcli --debug

# Ingestion debug mode
cirro graph ingest --type az --file cirro_output.db --debug
```

---

**Note**: Cirro is designed for authorized security testing and research. Ensure you have proper permissions before running against any cloud or network environment.
