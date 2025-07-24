# Cirro CLI Reference

The `cirro` command-line tool is the primary data collection component that gathers information from Azure Resource Manager (ARM) APIs and Microsoft Graph APIs.

## Synopsis

```bash
cirro <COMMAND> <AUTH_MODE> [OPTIONS]
```

## Commands

### `collect`

Collect data from Azure and Microsoft Graph APIs.

```bash
cirro collect <AUTH_MODE> [OPTIONS]
```

### `enrich`

Enrich data in the database with additional information.

```bash
cirro enrich <AUTH_MODE> [OPTIONS]
```

## Authentication Modes

### `access-token`

Authenticate using a pre-obtained access token.

```bash
cirro collect access-token --token <TOKEN> [OPTIONS]
cirro enrich access-token --token <TOKEN> [OPTIONS]
```

**Arguments:**

- `-t, --token <TOKEN>` - Access token for authentication

**Options:**

- `-o, --output-path <FILE>` - Output database file path (default: `cirro_output.db`)
- `--cloud <CLOUD>` - Cloud to enumerate (default: `public`)
- `--debug` - Enable debug output

### `azcli`

Authenticate using Azure CLI credentials.

```bash
cirro collect azcli [OPTIONS]
cirro enrich azcli [OPTIONS]
```

**Options:**

- `-t, --tenant-id <TENANT_ID>` - Tenant ID (optional)
- `-s, --subscription-id <SUBSCRIPTION_ID>` - Subscription ID (optional)
- `-o, --output-path <FILE>` - Output database file path (default: `cirro_output.db`)
- `-m, --mode <MODE>` - Enumeration mode (default: `both`)
- `--cloud <CLOUD>` - Cloud to enumerate (default: `public`)
- `--debug` - Enable debug output

### `client-secret`

Authenticate using a client secret.

```bash
cirro collect client-secret --client-id <ID> --client-secret <SECRET> --tenant-id <TENANT> [OPTIONS]
cirro enrich client-secret --client-id <ID> --client-secret <SECRET> --tenant-id <TENANT> [OPTIONS]
```

**Arguments:**

- `-c, --client-id <CLIENT_ID>` - Client ID
- `-p, --client-secret <CLIENT_SECRET>` - Client secret
- `-t, --tenant-id <TENANT_ID>` - Tenant ID

**Options:**

- `-o, --output-path <FILE>` - Output database file path (default: `cirro_output.db`)
- `-m, --mode <MODE>` - Enumeration mode (default: `both`)
- `--cloud <CLOUD>` - Cloud to enumerate (default: `public`)
- `--debug` - Enable debug output

### `client-cert`

Authenticate using a client certificate.

```bash
cirro collect client-cert --client-id <ID> --certificate <CERT_PATH> --tenant-id <TENANT> [OPTIONS]
cirro enrich client-cert --client-id <ID> --certificate <CERT_PATH> --tenant-id <TENANT> [OPTIONS]
```

**Arguments:**

- `-c, --client-id <CLIENT_ID>` - Client ID
- `-p, --certificate <FILE>` - Path to the client certificate file (PEM format)
- `-t, --tenant-id <TENANT_ID>` - Tenant ID

**Options:**

- `-o, --output-path <FILE>` - Output database file path (default: `cirro_output.db`)
- `-m, --mode <MODE>` - Enumeration mode (default: `both`)
- `--cloud <CLOUD>` - Cloud to enumerate (default: `public`)
- `--debug` - Enable debug output

### `user-pass`

Authenticate using username and password.

```bash
cirro collect user-pass --upn <UPN> --password <PASSWORD> [OPTIONS]
cirro enrich user-pass --upn <UPN> --password <PASSWORD> [OPTIONS]
```

**Arguments:**

- `-u, --upn <UPN>` - The username of the account (UPN format)
- `-p, --password <PASSWORD>` - The password of the account

**Options:**

- `-o, --output-path <FILE>` - Output database file path (default: `cirro_output.db`)
- `-m, --mode <MODE>` - Enumeration mode (default: `both`)
- `--cloud <CLOUD>` - Cloud to enumerate (default: `public`)
- `--debug` - Enable debug output

## Global Options

### Enumeration Mode (`--mode`)

Specifies which APIs to query during data collection.

- `both` (default) - Collect from both ARM and Graph APIs
- `arm` - Collect only from Azure Resource Manager APIs
- `graph` - Collect only from Microsoft Graph APIs

### Cloud Environment (`--cloud`)

Specifies the Azure cloud environment to target.

- `public` (default) - Azure Public Cloud
- `china` - Azure China Cloud
- `germany` - Azure Germany Cloud
- `usgov` - Azure US Government Cloud

### Output Path (`--output-path`)

Specifies the path for the output SQLite database file.

- Default: `cirro_output.db`
- Format: SQLite database file

### Debug Mode (`--debug`)

Enables detailed debug logging for troubleshooting.

- Default: `false`
- Provides verbose output for debugging authentication and API issues

## Enrichment Options

When using the `enrich` command, additional flags are available:

### `--storage-keys`

Gather storage account keys during enrichment.

```bash
cirro enrich azcli --storage-keys
```

## Examples

### Basic Collection

```bash
# Collect using Azure CLI authentication
cirro collect azcli

# Collect with specific tenant and subscription
cirro collect azcli --tenant-id "12345678-1234-1234-1234-123456789012" --subscription-id "87654321-4321-4321-4321-210987654321"

# Collect only ARM resources
cirro collect azcli --mode arm

# Collect only Graph data
cirro collect azcli --mode graph
```

### Authentication Examples

```bash
# Using client secret
cirro collect client-secret \
  --client-id "app-id-here" \
  --client-secret "secret-here" \
  --tenant-id "tenant-id-here"

# Using client certificate
cirro collect client-cert \
  --client-id "app-id-here" \
  --certificate "/path/to/cert.pem" \
  --tenant-id "tenant-id-here"

# Using access token
cirro collect access-token --token "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIs..."
```

### Cloud Environment Examples

```bash
# Collect from Azure US Government
cirro collect azcli --cloud usgov

# Collect from Azure China
cirro collect azcli --cloud china
```

### Data Enrichment

```bash
# Basic enrichment
cirro enrich azcli

# Enrichment with storage keys
cirro enrich azcli --storage-keys

# Enrichment with client secret
cirro enrich client-secret \
  --client-id "app-id-here" \
  --client-secret "secret-here" \
  --tenant-id "tenant-id-here" \
  --storage-keys
```

### Output and Debugging

```bash
# Custom output file
cirro collect azcli --output-path "/path/to/my-data.db"

# Enable debug logging
cirro collect azcli --debug

# Combined options
cirro collect azcli \
  --output-path "/path/to/my-data.db" \
  --mode both \
  --cloud public \
  --debug
```

## Notes

- Ensure you have appropriate permissions for the target Azure environment
- The access token authentication mode has limited options compared to other methods
- Debug mode provides detailed logging but may expose sensitive information
- The output database file will be created if it doesn't exist
- Some authentication modes require specific Azure AD application configurations