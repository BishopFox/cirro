# HAS_DISK

Represents disk attachment relationships in Azure compute resources.

**Direction:** `(resource)-[:HAS_DISK]->(disk)`

**Description:** Indicates that a resource (typically a Virtual Machine) has an attached disk for storage.

**Common Patterns:**

- Virtual machines have attached OS disks and data disks
- Disks can be managed or unmanaged
- Multiple VMs can potentially share disks (in specific configurations)

**Properties:** None

## Query Examples

```cypher
// Find all disks attached to virtual machines
MATCH path = (vm:VirtualMachine)-[:HAS_DISK]->(disk:Disk)
RETURN path

// Find virtual machines with multiple disks
MATCH (vm:VirtualMachine)-[:HAS_DISK]->(disk:Disk)
WITH vm, COUNT(disk) as diskCount
WHERE diskCount > 1
RETURN vm, diskCount
ORDER BY diskCount DESC

// Find disk usage patterns and sizes
MATCH (vm:VirtualMachine)-[:HAS_DISK]->(disk:Disk)
RETURN vm.name, disk.name, disk.diskSizeGB, disk.osType, disk.diskState

// Find large disks
MATCH (vm)-[:HAS_DISK]->(disk:Disk)
WHERE disk.diskSizeGB > 1000
RETURN vm, disk, disk.diskSizeGB
ORDER BY disk.diskSizeGB DESC
```
