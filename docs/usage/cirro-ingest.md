# Cirro-Ingest CLI Reference

The `cirro-ingest` command-line tool is responsible for loading data collected by `cirro` into graph databases (Neo4j or Memgraph).

!!! warning "Security Notice"

    - **Passwords**: Avoid using default passwords in production
    - **TLS**: Use encrypted connections (`bolt+s://`, `neo4j+s://`) for remote servers
    - **Credentials**: Store credentials securely (environment variables, credential managers)
    - **Network**: Ensure database servers are properly secured and firewalled

## Synopsis

The `cirro-ingest` tool performs the following operations:

1. **Database Connection** - Establishes connection to the target graph database
2. **Schema Setup** - Creates necessary constraints and indexes
3. **Data Transformation** - Converts SQLite data to graph format
4. **Node Creation** - Creates nodes for all collected entities
5. **Relationship Creation** - Establishes relationships between entities
6. **Index Optimization** - Optimizes database performance

```bash
cirro-ingest --file <DATABASE_FILE> [OPTIONS]
```

## Required Arguments

`--file` / `-f`

Path to the Cirro results database file (SQLite) created by the `cirro` tool.

```bash
cirro-ingest --file cirro_output.db
```

**Format:** SQLite database file path  
**Example:** `cirro_output.db`, `/path/to/results.db`

## Options

`--graph-type` / `-g`

Specifies the target graph database type.

```bash
cirro-ingest --file cirro_output.db --graph-type neo4j
```

**Values:**

- `neo4j` (default) - Neo4j graph database
- `memgraph` - Memgraph graph database

**Default:** `neo4j`

`--server` / `-s`

Database server connection string with supported URI schemes.

```bash
cirro-ingest --file cirro_output.db --server bolt://localhost:7687
```

**Supported Schemes:**

- `bolt://` - Plain Bolt protocol
- `bolt+s://` - Bolt with TLS encryption
- `bolt+ssc://` - Bolt with self-signed certificate
- `neo4j://` - Neo4j routing protocol
- `neo4j+s://` - Neo4j routing with TLS encryption
- `neo4j+ssc://` - Neo4j routing with self-signed certificate

**Default:** `bolt://localhost:7687`

`--user` / `-u`

Database username for authentication.

```bash
cirro-ingest --file cirro_output.db --user myuser
```

**Default:** `neo4j`

`--password` / `-p`

Database password for authentication.

```bash
cirro-ingest --file cirro_output.db --password mypassword
```

**Default:** `password`

`--db-name` / `-d`

Database name to connect to (optional).

```bash
cirro-ingest --file cirro_output.db --db-name mydatabase
```

**Default Behavior:**

- For Neo4j: defaults to `neo4j`
- For Memgraph: defaults to `memgraph`

`--debug`

Enable debug logging for troubleshooting.

```bash
cirro-ingest --file cirro_output.db --debug
```

**Default:** `false`

## Examples

### Basic Usage

```bash
# Basic ingestion to Neo4j with defaults
cirro-ingest --file cirro_output.db

# Basic ingestion to Memgraph
cirro-ingest --file cirro_output.db --graph-type memgraph
```

### Neo4j Examples

```bash
# Neo4j with custom server
cirro-ingest \
  --file cirro_output.db \
  --graph-type neo4j \
  --server bolt://neo4j.example.com:7687 \
  --user neo4j \
  --password mypassword

# Neo4j with TLS encryption
cirro-ingest \
  --file cirro_output.db \
  --server bolt+s://neo4j.example.com:7687 \
  --user neo4j \
  --password mypassword

# Neo4j cluster with routing
cirro-ingest \
  --file cirro_output.db \
  --server neo4j://neo4j-cluster.example.com:7687 \
  --user neo4j \
  --password mypassword

# Neo4j with custom database name
cirro-ingest \
  --file cirro_output.db \
  --db-name analytics \
  --user neo4j \
  --password mypassword
```

### Memgraph Examples

```bash
# Memgraph with default settings
cirro-ingest \
  --file cirro_output.db \
  --graph-type memgraph \
  --server bolt://localhost:7687 \
  --user memgraph \
  --password memgraph

# Memgraph with custom server
cirro-ingest \
  --file cirro_output.db \
  --graph-type memgraph \
  --server bolt://memgraph.example.com:7687 \
  --user admin \
  --password secret

# Memgraph with TLS
cirro-ingest \
  --file cirro_output.db \
  --graph-type memgraph \
  --server bolt+s://memgraph.example.com:7687 \
  --user admin \
  --password secret
```

### Debug and Troubleshooting

```bash
# Enable debug logging
cirro-ingest --file cirro_output.db --debug

# Full example with all options
cirro-ingest \
  --file /path/to/cirro_output.db \
  --graph-type neo4j \
  --server bolt+s://neo4j.example.com:7687 \
  --user myuser \
  --password mypassword \
  --db-name mydb \
  --debug
```

## Connection Examples

### Local Development

```bash
# Local Neo4j (default)
cirro-ingest --file cirro_output.db

# Local Memgraph
cirro-ingest \
  --file cirro_output.db \
  --graph-type memgraph \
  --user memgraph \
  --password memgraph
```

### Docker Containers

```bash
# Neo4j in Docker
cirro-ingest \
  --file cirro_output.db \
  --server bolt://localhost:7687 \
  --user neo4j \
  --password password

# Memgraph in Docker
cirro-ingest \
  --file cirro_output.db \
  --graph-type memgraph \
  --server bolt://localhost:7687 \
  --user memgraph \
  --password memgraph
```

### Cloud Instances

```bash
# Neo4j AuraDB (cloud)
cirro-ingest \
  --file cirro_output.db \
  --server neo4j+s://xxxxxxxx.databases.neo4j.io:7687 \
  --user neo4j \
  --password your-aura-password

# Self-hosted with TLS
cirro-ingest \
  --file cirro_output.db \
  --server bolt+s://your-server.com:7687 \
  --user your-username \
  --password your-password
```

