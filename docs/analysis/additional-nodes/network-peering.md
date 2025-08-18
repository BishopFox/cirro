# NetworkPeering

Represents virtual network peering connections.

**Labels:** `:NetworkPeering`

**Properties:**

- `id` - Peering ID (primary key)
- `name` - Peering name
- `type` - Peering type
- `allowForwardedTraffic` - Whether forwarded traffic is allowed
- `allowGatewayTransit` - Whether gateway transit is allowed
- `allowVirtualNetworkAccess` - Whether virtual network access is allowed
- `doNotVerifyRemoteGateway` - Whether to skip remote gateway verification
- `peerCompleteVnets` - Whether to peer complete VNets
- `peeringState` - Peering state
- `peeringSyncLevel` - Peering sync level
- `remoteAddressPrefixes` - Remote address prefixes
- `useRemoteGateways` - Whether to use remote gateways

**Relationships:**
- `HAS_PEERING` ← VirtualNetwork
- `HAS_GATEWAY` → NetworkGateway
- `PEER_TO` → VirtualNetwork
