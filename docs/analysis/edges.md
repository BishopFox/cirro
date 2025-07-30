# Relationship Types (Edges)

This document describes all the relationship types created by Cirro's data ingestion process and their meanings.

## Microsoft Graph Relationships

### MEMBER_OF

Represents membership relationships between Graph objects.

**Direction:** `(user/device/object) -[:MEMBER_OF]-> (group/administrativeUnit)`

**Description:** Indicates that a user, device, or other object is a member of a group or administrative unit.

**Common Patterns:**
- Users are members of groups
- Devices are members of groups
- Objects are members of administrative units

**Properties:** None

### OWNS

Represents ownership relationships in Azure AD.

**Direction:** `(user/servicePrincipal) -[:OWNS]-> (application/group/servicePrincipal)`

**Description:** Indicates that a user or service principal owns an application, group, or other service principal.

**Common Patterns:**
- Users own applications they created
- Service principals own other service principals
- Users own groups they manage

**Properties:** None

### HAS_ROLE

Represents role assignments in Azure AD.

**Direction:** `(user/servicePrincipal) -[:HAS_ROLE]-> (role/administrativeUnit)`

**Description:** Indicates that a user or service principal has been assigned a specific role or role within an administrative unit.

**Common Patterns:**
- Users have directory roles (Global Administrator, User Administrator, etc.)
- Service principals have role assignments
- Objects have scoped roles within administrative units

**Properties:**
- `roleId` - ID of the assigned role
- `roleName` - Display name of the role (when available)

### REGISTERED_USER

Represents device registration relationships.

**Direction:** `(user) -[:REGISTERED_USER]-> (device)`

**Description:** Indicates that a user is registered as a user of a device.

**Properties:** None

### REGISTERED_OWNER

Represents device ownership relationships.

**Direction:** `(user) -[:REGISTERED_OWNER]-> (device)`

**Description:** Indicates that a user is registered as an owner of a device.

**Properties:** None

### REPRESENTED_BY

Represents the relationship between applications and service principals.

**Direction:** `(application) -[:REPRESENTED_BY]-> (servicePrincipal)`

**Description:** Indicates that an application is represented by a service principal in the directory.

**Properties:** None

### AUTHENTICATES

Represents authentication credential relationships.

**Direction:** `(clientSecret/certificate) -[:AUTHENTICATES]-> (application/servicePrincipal)`

**Description:** Indicates that a client secret or certificate can be used to authenticate as an application or service principal.

**Properties:** None

### ASSOCIATED_WITH

Represents bidirectional associations between related entities.

**Direction:** `(tenant) -[:ASSOCIATED_WITH]-> (graphOrg)` and `(graphOrg) -[:ASSOCIATED_WITH]-> (tenant)`

**Description:** Creates a bidirectional link between Azure tenant information (from ARM API) and organizational details (from Microsoft Graph API).

**Common Patterns:**
- Tenants are associated with their Graph organizations
- Graph organizations are associated with their corresponding tenants

**Properties:** None

### VERIFIED_DOMAIN

Represents domain verification relationships.

**Direction:** `(graphOrg) -[:VERIFIED_DOMAIN]-> (verifiedDomain)`

**Description:** Indicates that an Azure AD organization has verified ownership of a domain.

**Common Patterns:**
- Organizations have multiple verified domains
- Default domains are used for new user creation
- Initial domains are automatically created with tenants

**Properties:** None

## Azure Resource Manager Relationships

### CONTAINS

Represents hierarchical containment relationships in Azure.

**Direction:** `(parent) -[:CONTAINS]-> (child)`

**Description:** Indicates that one Azure resource contains or manages another resource.

**Common Patterns:**
- Tenants contain subscriptions
- Subscriptions contain resource groups
- Resource groups contain resources
- Virtual networks contain subnets

**Properties:** None

### CONNECTED_TO

Represents network connectivity relationships.

**Direction:** `(resource) -[:CONNECTED_TO]-> (networkResource)`

**Description:** Indicates that a resource is connected to a network resource.

**Common Patterns:**
- Virtual machines connected to network interfaces
- Network interfaces connected to subnets
- Network interfaces connected to network security groups

**Properties:** None

### USES

Represents resource usage relationships.

**Direction:** `(resource) -[:USES]-> (usedResource)`

**Description:** Indicates that one resource uses or depends on another resource.

**Common Patterns:**
- Virtual machines use disks
- Virtual machines use availability sets
- Resources use managed identities

**Properties:** None

### LOCATED_IN

Represents location or placement relationships.

**Direction:** `(resource) -[:LOCATED_IN]-> (container)`

**Description:** Indicates that a resource is located within or placed in another resource.

**Common Patterns:**
- Resources located in resource groups
- Virtual machines located in availability sets
- Network interfaces located in subnets

**Properties:** None

### MANAGES

Represents management relationships between resources.

**Direction:** `(manager) -[:MANAGES]-> (managed)`

**Description:** Indicates that one resource manages or controls another resource.

**Common Patterns:**
- Automation accounts manage runbooks
- SQL servers manage databases

**Properties:** None

## RBAC (Role-Based Access Control) Relationships

### Dynamic Role Relationships

RBAC relationships are created dynamically based on Azure role assignments. The relationship type matches the role name.

**Direction:** `(principal) -[:ROLE_NAME]-> (scope)`

**Description:** Indicates that a security principal (user, group, service principal) has been assigned a specific role at a particular scope.

**Common Role Types:**
- `Owner` - Full access to all resources
- `Contributor` - Can create and manage all types of Azure resources
- `Reader` - Can view existing Azure resources
- `User Access Administrator` - Can manage user access to Azure resources
- `Security Admin` - Can manage security policies and settings
- Various service-specific roles (e.g., `Storage Account Contributor`, `Virtual Machine Contributor`)

**Properties:**
- `id` - Role assignment ID
- `description` - Role assignment description
- `roleName` - Name of the assigned role
- `roleType` - Type of role (BuiltInRole, CustomRole)
- `actions` - Array of allowed actions
- `notActions` - Array of denied actions
- `dataActions` - Array of allowed data actions
- `notDataActions` - Array of denied data actions

**Examples:**
```cypher
(user:GraphUser) -[:Owner]-> (subscription:Subscription)
(servicePrincipal:GraphServicePrincipal) -[:Contributor]-> (resourceGroup:ResourceGroup)
(group:GraphGroup) -[:Reader]-> (storageAccount:StorageAccount)
```

## Security and Access Relationships

### HAS_ACCESS_POLICY

Represents Key Vault access policy relationships.

**Direction:** `(principal) -[:HAS_ACCESS_POLICY]-> (keyVault)`

**Description:** Indicates that a security principal has an access policy defined for a Key Vault.

**Properties:**
- Permissions for keys, secrets, and certificates
- Access policy details

### ASSIGNED_TO

Represents general assignment relationships.

**Direction:** `(principal) -[:ASSIGNED_TO]-> (resource/role)`

**Description:** Generic relationship indicating that a principal is assigned to a resource or role.

**Properties:** Varies based on assignment type

## Network Relationships

### ALLOWS / DENIES

Represents network security group rule relationships.

**Direction:** `(nsgRule) -[:ALLOWS/DENIES]-> (traffic)`

**Description:** Indicates what traffic a network security group rule allows or denies.

**Properties:**
- Traffic direction
- Source and destination specifications
- Port ranges
- Protocols

### PROTECTED_BY

Represents security protection relationships.

**Direction:** `(resource) -[:PROTECTED_BY]-> (securityResource)`

**Description:** Indicates that a resource is protected by a security resource.

**Common Patterns:**
- Network interfaces protected by network security groups
- Subnets protected by network security groups

**Properties:** None

## Storage Relationships

### HAS_CONTAINER

Represents storage container relationships.

**Direction:** `(storageAccount) -[:HAS_CONTAINER]-> (container)`

**Description:** Indicates that a storage account contains a specific container.

**Properties:** Container access level and properties

### HAS_BLOB

Represents blob storage relationships.

**Direction:** `(container) -[:HAS_BLOB]-> (blob)`

**Description:** Indicates that a container contains a specific blob.

**Properties:** Blob properties and metadata

## Enrichment Relationships

### HAS_KEY

Represents storage account key relationships (from enrichment).

**Direction:** `(storageAccount) -[:HAS_KEY]-> (key)`

**Description:** Indicates that a storage account has access keys (discovered during enrichment).

**Properties:**
- Key name
- Key permissions
- Key type

## Relationship Properties

Many relationships include additional properties that provide context:

- **Timestamps** - When relationships were created or modified
- **Permissions** - What permissions are granted
- **Configuration** - Specific configuration details
- **Metadata** - Additional metadata from Azure APIs

## Notes

- Relationships are automatically deduplicated during post-processing
- Self-relationships (loops) are handled appropriately by the ingestion process
- Some relationships are bidirectional in practice but stored as directed edges
- Relationship properties may vary based on the specific Azure service and configuration
- Additional relationships may be created during enrichment processes

## Query Examples

```cypher
-- Find all users with Owner roles
MATCH (u:GraphUser) -[:Owner]-> (r)
RETURN u, r

-- Find network security relationships
MATCH (vm:VirtualMachine) -[:CONNECTED_TO]-> (nic:NetworkInterface) -[:PROTECTED_BY]-> (nsg:NetworkSecurityGroup)
RETURN vm, nic, nsg

-- Find group membership chains
MATCH path = (u:GraphUser) -[:MEMBER_OF*1..3]-> (g:GraphGroup)
RETURN path

-- Find application ownership
MATCH (owner) -[:OWNS]-> (app:GraphApplication) <-[:REPRESENTED_BY]- (sp:GraphServicePrincipal)
RETURN owner, app, sp
```