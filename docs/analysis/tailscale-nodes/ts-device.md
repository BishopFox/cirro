# TSDevice

Represents a device connected to a Tailscale network, including both the local device and peer devices.

**Labels:** `:TSDevice`

**Properties:**

- `id` - Unique device identifier (primary key)
- `publicKey` - Device's public key for authentication
- `hostname` - Device hostname
- `dnsName` - DNS name assigned by Tailscale
- `certDomains` - Certificate domains (for self device)
- `os` - Operating system of the device
- `tailscaleIPs` - Array of Tailscale IP addresses assigned to the device
- `allowedIPs` - Array of IP addresses/ranges the device is allowed to access
- `primaryRoutes` - Primary routes advertised by the device (peer devices only)
- `relay` - Relay server used for connection
- `created` - Timestamp when the device was created
- `lastWrite` - Timestamp of last write operation
- `lastSeen` - Timestamp when the device was last seen online
- `lastHandshake` - Timestamp of last cryptographic handshake
- `noFileSharingReason` - Reason why file sharing is not available (if applicable)
- `online` - Boolean indicating if the device is currently online
- `exitNode` - Boolean indicating if the device is configured as an exit node
- `active` - Boolean indicating if the device is active
- `keyExpiry` - Timestamp when the device's key expires
- `sshHostKeys` - SSH host keys for the device (peer devices only)
- `tags` - Array of tags assigned to the device

## Relationships

- **TSTailnet** → **CONTAINS** → **TSDevice** - Tailnets contain devices
- **TSUser** → **OWNS** → **TSDevice** - Users own the devices they register