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

## Notes

- Ensure you have appropriate permissions for your target environment.
- Debug logging can include sensitive values; handle output carefully.