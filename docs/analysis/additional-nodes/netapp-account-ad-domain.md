# NetAppAccountADDomain

Represents an Active Directory domain configuration for an Azure NetApp Files account.

**Labels:** `:NetAppAccountADDomain`

**Properties:**

- `activeDirectoryId` - Active Directory identifier (primary key)
- `administrators` - List of administrator accounts
- `aesEncryption` - Whether AES encryption is enabled
- `allowLocalNfsUsersWithLdap` - Whether local NFS users are allowed with LDAP
- `backupOperators` - Backup operator accounts for the AD domain
- `dns` - DNS server configuration
- `domain` - Active Directory domain name
- `encryptDCConnections` - Whether domain controller connections are encrypted
- `kdcIP` - Kerberos KDC IP address
- `ldapOverTLS` - Whether LDAP over TLS is enabled
- `ldapSigning` - LDAP signing configuration
- `organizationalUnit` - Organizational unit for computer accounts
- `site` - Active Directory site name
- `smbServerName` - SMB server name
- `status` - Current status of the AD configuration
- `username` - Username for domain authentication

## Relationships

- **NetAppAccount** → **IS_DEPLOYED** → **NetAppAccountADDomain** - AD domain where NetApp is deployed