# ComputeImage

Represents a compute image resource that can be used to create virtual machines.

**Labels:** `:ArmResource`, `:ComputeImage`

**Properties:**

- `id` - Resource identifier (primary key)
- `hyperVGeneration` - Hyper-V generation of the image
- `provisioningState` - Current provisioning state
- `blobUri` - URI of the blob containing the image (when applicable)
- `osType` - Operating system type (Linux/Windows)
- `osState` - Operating system state (Generalized/Specialized)
- `diskSizeGB` - Size of the disk in gigabytes

## Relationships

- **HAS_SOURCE** → **VirtualMachine** - Links to the source virtual machine if the image was created from a VM