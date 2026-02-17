# ENCRYPTED_BY

Represents encryption of disks by a Disk Encryption Set.

## Usage

- **Disk** → `ENCRYPTED_BY` → **DiskEncryptionSet**

## Examples

```cypher
MATCH (d:Disk)-[:ENCRYPTED_BY]->(des:DiskEncryptionSet)
RETURN d.id, des.activeKey, des.encryptionType
```
