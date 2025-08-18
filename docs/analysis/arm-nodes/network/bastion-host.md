# BastionHost

Represents Azure Bastion hosts.

**Labels:** `:ArmResource:BastionHost`

**Properties:**

- `id` - Bastion host resource ID (primary key)
- `disableCopyPaste` - Whether copy/paste is disabled
- `dnsName` - DNS name
- `enableIpConnect` - Whether IP connect is enabled
- `enableKerberos` - Whether Kerberos is enabled
- `enablePrivateOnlyBastion` - Whether private-only bastion is enabled
- `enableSessionRecording` - Whether session recording is enabled
- `enableShareableLink` - Whether shareable link is enabled
- `enableTunneling` - Whether tunneling is enabled
- `scaleUnits` - Number of scale units

**Relationships:**
- `HAS_CONFIG` → BastionIPConfig
