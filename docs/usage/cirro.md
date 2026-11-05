# Data Collection

Cirro's collection functionality gathers data from various cloud platforms and services.

## Command Structure

```bash
cirro collect <platform> <auth-method> [options]
```

## Platforms and Auth Methods

### Azure

```bash
cirro collect az azcli [options]
cirro collect az client-secret --client-id <ID> --client-secret <SECRET> --tenant-id <TENANT> [options]
cirro collect az client-cert --client-id <ID> --certificate <CERT_PATH> --tenant-id <TENANT> [options]
cirro collect az access-token --token <TOKEN> [options]
cirro collect az user-pass --upn <UPN> --password <PASSWORD> [options]
```

### Tailscale

```bash
cirro collect ts <auth-method> [options]
```

## Examples

```bash
# Azure via Azure CLI
cirro collect az azcli --tenant-id <tenant-id> --mode both

# Azure via client secret
cirro collect az client-secret \
  --client-id <id> \
  --client-secret <secret> \
  --tenant-id <tenant> \
  --cloud usgov

# Tailscale collection
cirro collect ts <auth-method> --output-path cirro_ts_socket.json

# Debug mode
cirro collect az azcli --debug
```

## Restricted Guest Collection

Use `--restricted-guest` to collect Microsoft Graph data as a restricted Entra ID guest. These options are available with all Azure authentication commands:

| Option | Usage |
| --- | --- |
| `--restricted-guest` | Enable restricted guest collection using the default application IDs. |
| `--restricted-appids <GUID> ...` | Replace the default application/client IDs. Accepts comma-separated, space-separated or repeated values. |
| `--restricted-guest-msapps` | Use the Microsoft application catalog. Prompts for confirmation before collection. |

`--restricted-appids` and `--restricted-guest-msapps` both require `--restricted-guest` and cannot be combined.

```bash
# Use the default application IDs
cirro collect az azcli --tenant-id <tenant-id> --mode graph --restricted-guest \
  --output-path cirro_restricted.db

# Use an existing Microsoft Graph token
cirro collect az access-token --token <GRAPH_ACCESS_TOKEN> --restricted-guest \
  --output-path cirro_restricted.db

# Use a specific application ID
cirro collect az azcli --tenant-id <tenant-id> --mode graph --restricted-guest \
  --restricted-appids <APP_ID>

# Use the Microsoft application catalog
cirro collect az azcli --tenant-id <tenant-id> --mode graph \
  --restricted-guest --restricted-guest-msapps
```

Use `--mode graph` for Graph-only collection or `--mode both` to include ARM. Restricted guest options are ignored in `--mode arm`. With `access-token`, supply a Microsoft Graph token and omit `--mode`.

Normal Graph selection flags such as `--users`, `--no-default`, `--caps` and `--graph-pim` do not apply in restricted guest mode. Ingest the output using the normal [Azure ingestion command](cirro-graph.md#restricted-guest-results).

## Notes

- Ensure you have appropriate permissions for your target environment.
- Debug logging can include sensitive values; handle output carefully.
