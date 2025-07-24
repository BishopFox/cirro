# NeoDash Dashboard

NeoDash is a low-code dashboard builder that allows you to create interactive dashboards to visualize and analyze your Cirro graph data. The Cirro project includes a pre-configured NeoDash dashboard with common security analysis queries.

## Dashboard Overview

The Cirro NeoDash dashboard provides multiple pages focused on different aspects of Azure and Entra ID security analysis:

- **Entra ID Admins** - Administrative role analysis
- **Entra ID Groups** - Group membership and ownership analysis  
- **Entra ID Devices** - Device registration and compliance
- **Entra ID Apps and SPNs** - Application and service principal analysis
- **Entra ID Owned Objects** - Object ownership relationships
- **Subscriptions** - Azure subscription permissions
- **Key Vaults** - Key Vault access analysis
- **Storage Accounts** - Storage security configuration
- **Virtual Machines** - VM network security analysis

## Getting Started

### Prerequisites

1. **Neo4j/Memgraph Database** with Cirro data loaded
2. **NeoDash** deployed and accessible
3. **Network connectivity** between user and database

### Loading the Dashboard

1. Import the dashboard configuration from `tools/neodash_config.json`
2. Configure the Neo4j connection settings
3. Verify data connectivity and refresh the reports

## Dashboard Pages

### Entra ID Admins

This page focuses on identifying users with administrative privileges in Entra ID.

#### Global Administrators

Identifies all users with Global Administrator roles, both through direct assignment and group inheritance.

```cypher
// Direct Role Assignment
MATCH (o:GraphObject)-[:HAS_ROLE]->(g:GraphRole)
WHERE g.displayName = "Global Administrator"
RETURN 
  o.displayName AS MemberName,
  o.userPrincipalName AS UPN,
  labels(o)[1] AS Type,
  null AS GroupName, 
  g.displayName AS RoleName

UNION

// Group-Inherited Role Assignment
MATCH (o:GraphObject)-[:MEMBER_OF*1..]->(group:GraphObject)-[:HAS_ROLE]->(g:GraphRole)
WHERE g.displayName = "Global Administrator"
RETURN 
  o.displayName AS MemberName,
  o.userPrincipalName AS UPN,
  labels(o)[1] AS Type,
  group.displayName AS GroupName, 
  g.displayName AS RoleName
```

#### Privileged Administrator Roles

Identifies users with high-privilege administrative roles including Global Administrator, User Administrator, and other critical roles.

```cypher
// Direct Role Assignment
MATCH (o:GraphObject)-[:HAS_ROLE]->(g:GraphRole)
WHERE g.displayName =~ 'Global Administrator|User Administrator|Cloud Application Administrator|Authentication Policy Administrator|Exchange Administrator|Helpdesk Administrator|Privileged Authentication Administrator'
RETURN 
  o.displayName AS MemberName,
  o.userPrincipalName AS UPN,
  labels(o)[1] AS Type,
  null AS GroupName, 
  g.displayName AS RoleName

UNION

// Group-Inherited Role Assignment
MATCH (o:GraphObject)-[:MEMBER_OF*..]->(group:GraphGroup)-[:HAS_ROLE]->(g:GraphRole)
WHERE g.displayName =~ 'Global Administrator|User Administrator|Cloud Application Administrator|Authentication Policy Administrator|Exchange Administrator|Helpdesk Administrator|Privileged Authentication Administrator'
RETURN 
  o.displayName AS MemberName,
  o.userPrincipalName AS UPN,
  labels(o)[1] AS Type,
  group.displayName AS GroupName, 
  g.displayName AS RoleName
```

### Entra ID Groups

Analyzes group configurations and potential security issues.

#### Dynamic Groups Without Membership Check

Identifies dynamic groups that don't include user type validation, which could lead to unintended access.

```cypher
MATCH (n:GraphGroup)
WHERE n.membershipRule IS NOT NULL AND NOT n.membershipRule CONTAINS "user.userType"
RETURN n.displayName, n.membershipRuleProcessingState, n.membershipRule
```

#### Group Ownership

Shows who owns each group, which is important for access governance.

```cypher
MATCH (o:GraphObject)-[:OWNS]->(t:GraphGroup)
RETURN 
  labels(o)[1] AS OwnerType,
  o.displayName AS Owner,
  o.userPrincipalName AS UPN,
  t.displayName AS OwnedName
```

#### Circular Group Membership

Detects circular group memberships that can cause issues in access evaluation.

```cypher
MATCH p=(a:GraphGroup)-[:MEMBER_OF*2..]->(b:GraphGroup)
WHERE a.id = b.id
RETURN p
```

### Entra ID Devices

Analyzes device registration and compliance status.

#### Device Registration Analysis

Comprehensive view of device registrations showing user relationships and device properties.

```cypher
MATCH (d:GraphDevice)
WHERE d.accountEnabled = true
OPTIONAL MATCH (o)-[r:REGISTERED_USER|REGISTERED_OWNER]->(d)
WITH d, o,
     collect(DISTINCT TYPE(r)) AS relTypes
WITH d, o,
     CASE
       WHEN "REGISTERED_USER" IN relTypes AND "REGISTERED_OWNER" IN relTypes THEN "REGISTERED_USER | REGISTERED_OWNER"
       WHEN "REGISTERED_USER" IN relTypes THEN "REGISTERED_USER"
       WHEN "REGISTERED_OWNER" IN relTypes THEN "REGISTERED_OWNER"
       ELSE null
     END AS RelationshipType
RETURN
  o.userPrincipalName AS UPN,
  RelationshipType,
  d.deviceId AS DeviceId,
  d.displayName AS DeviceName,
  d.profileType AS ProfileType,
  d.trustType AS TrustType,
  d.isCompliant AS Compliant,
  d.operatingSystem AS OS,
  d.operatingSystemVersion AS OSVersion
```

### Entra ID Apps and SPNs

Analyzes application and service principal security configurations.

#### Secrets and Certificates

Shows authentication credentials for applications and service principals.

```cypher
MATCH (a:GraphApplication|GraphServicePrincipal)
MATCH (s:ClientSecret|Certificate)-[:AUTHENTICATES]->(a)
RETURN
  a.id AS Id,
  a.displayName AS DisplayName,
  labels(s)[0] AS CredType,
  s.displayName AS CredName,
  s.hint AS ClientSecretHint,
  s.type AS CertificateType,
  s.thumbprint AS CertificateThumbprint,
  s.startDateTime AS CertificateStartTime,
  s.endDateTime AS CertificateEndTime
```

#### Managed Identities

Analyzes managed identity role assignments and resource access.

```cypher
MATCH (o:ArmResource)-[r:HAS_IDENTITY]->(i:GraphServicePrincipal)-[p]->(a:ArmResource)
WHERE p.roleName IS NOT NULL
RETURN
  o.id AS ArmResource,
  o.name AS Name,
  p.roleName AS RoleName,
  a.name AS ResourceName,
  a.id AS ResourceId
```

### Subscriptions

Analyzes Azure subscription-level permissions and access patterns.

#### Direct Permissions to Subscriptions

Shows users and service principals with direct subscription-level access.

```cypher
MATCH (o)-[r]->(s:Subscription)
WHERE r.roleName is NOT NULL
RETURN
  o.displayName AS displayName,
  o.userPrincipalName AS UPN,
  r.roleName AS role,
  s.subscriptionId AS Id,
  s.displayName AS Name
```

#### Direct Permissions to Resource Groups

Identifies permissions assigned at the resource group level.

```cypher
MATCH (o)-[r]->(g:ResourceGroup)
MATCH (s:Subscription)-[c]->(g)
WHERE r.roleName is NOT NULL
RETURN
  o.displayName,
  o.userPrincipalName AS UPN,
  r.roleName,
  g.name AS RgName,
  s.displayName AS SubName
```

### Key Vaults

Analyzes Key Vault access patterns and security configurations.

#### Direct Key Vault Permissions

Shows direct RBAC permissions to Key Vaults.

```cypher
MATCH (n:GraphObject)-[r]->(v:KeyVault)
RETURN 
  n.id as id,
  labels(n)[0] as type,
  n.displayName as displayName,
  r.roleName AS roleName,
  v.id AS vaultId
```

#### Transitive Key Vault Paths

Identifies indirect access paths to Key Vaults through group memberships or nested permissions.

```cypher
MATCH (n:GraphObject)-[r*1..5]->(v:KeyVault)
WHERE ANY(rel IN r WHERE rel.roleType IS NOT NULL)
RETURN 
  n.id AS id, 
  labels(n)[0] AS type, 
  n.displayName AS displayName, 
  v.id AS vaultId,  
  [rel IN r | 
    type(rel) + "->" + coalesce(endNode(rel).displayName, endNode(rel).name, "Unnamed")
  ] AS relationshipTypesWithDirection
```

### Storage Accounts

Analyzes storage account security configurations.

#### Storage Accounts with Public Network Access

Identifies storage accounts with potentially insecure public access configurations.

```cypher
MATCH (n:StorageAccount) 
WHERE n.allowBlobPublicAccess = true AND n.publicNetworkAccess = "Enabled" 
RETURN n.id AS id, n.name AS name, n.supportsHttpsTrafficOnly as httpsOnly
```

#### Storage Account Keys

Shows storage account access keys (when enrichment has been performed).

```cypher
MATCH (s:StorageAccount)-[:HAS_KEY]->(k:StorageAccountKey)
RETURN
   s.name as StorageName,
   k.name as KeyName,
   k.value as KeyValue
```

### Virtual Machines

Analyzes virtual machine network security configurations.

#### Virtual Machines with Public IPs

Complex analysis showing VMs with public IP addresses and their network security group rules.

```cypher
MATCH (vm:VirtualMachine)-[:HAS_NIC]->(ni:NetworkInterface)-[:HAS_NSG]->(:NSG)-[:HAS_RULE]-(rule:NSGRule)
WHERE rule.access = "Allow" AND NOT toLower(rule.type) CONTAINS "defaultsecurityrules"
OPTIONAL MATCH (ni)-[:HAS_IPCONFIG]->(ipcon:IpConfiguration)-[:HAS_IP]->(ip:PublicIPAddress)
OPTIONAL MATCH (vn:VirtualNetwork)-[:CONTAINS]->(sub:Subnet)-[:CONTAINS]->(ipcon)
WITH DISTINCT vm, rule, vn, sub,
     COLLECT(DISTINCT ipcon.privateIPAddress) AS privateIps, 
     COLLECT(DISTINCT ip.ipAddress) AS publicIps,
     COLLECT(DISTINCT rule.sourceAddressPrefix) AS sourceAddressPrefix,
     COLLECT(DISTINCT rule.sourcePortRange) AS sourcePorts,
     COLLECT(DISTINCT rule.sourcePortRanges) AS sourcePortRanges,
     COLLECT(DISTINCT rule.destinationAddressPrefix) as destinationAddressPrefix,
     COLLECT(DISTINCT rule.destinationPortRange) AS destinationPorts,
     COLLECT(DISTINCT rule.destinationPortRanges) AS destinationPortRanges
RETURN vm.name AS name,
       vm.os AS os,
       rule.name AS ruleName,
       rule.access AS access,
       rule.direction AS direction,
       vn.name AS virtualNetworkName,
       sub.addressPrefix as addressPrefix,
       privateIps[0] AS privateIp, 
       publicIps[0] AS publicIP,
       sourceAddressPrefix,
       rule.sourceAddressPrefixes AS sourceAddressPrefixes,
       sourcePorts,
       sourcePortRanges,
       destinationAddressPrefix,
       rule.destinationAddressPrefixes AS destinationAddressPrefixes,
       destinationPorts,
       destinationPortRanges
```
## Extending the Dashboard

### Adding New Reports

1. **Identify security use cases** requiring visualization
2. **Develop and test queries** in Neo4j Browser first
3. **Create new dashboard reports** with appropriate visualizations
4. **Document query logic** and expected results

### Custom Query Patterns

Common patterns for extending the dashboard:

```cypher
// Pattern: Find privilege escalation paths
MATCH path = (start:GraphUser)-[*1..5]->(end:GraphRole)
WHERE start.userType = "Member" AND end.displayName CONTAINS "Administrator"
RETURN path

// Pattern: Identify privileged service principals
MATCH path=(sp:GraphServicePrincipal)-[r]->(resource)
WHERE r.roleName IN ["Owner", "Contributor"]
RETURN path

```