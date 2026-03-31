# MOUNTS

Links storage volumes to their mount targets or mount configurations.

**Relationship:** `:MOUNTS`

**Direction:** Various (depends on the specific resource types being linked)

## Description

This relationship represents mounting relationships where storage volumes are made available through specific mount points or targets.

## Examples

### NetAppVolume → NetAppVolumeMount
Links NetApp volumes to their mount targets, which provide the network endpoints and configuration needed to access the volume from client systems.

## Use Cases

- Understanding how storage volumes are accessed and mounted
- Mapping network endpoints for file system access
- Identifying mount configurations and access paths for troubleshooting