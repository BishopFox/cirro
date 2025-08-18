# SqlVirtualMachine

Azure SQL Virtual Machine resources that configure SQL Server settings on Azure virtual machines.

**Labels:** `:ArmResource:SqlVirtualMachine`

## Properties

- `id` - Resource ID (inherited from ArmResource)
- `name` - Resource name (inherited from ArmResource)
- `type` - Resource type (inherited from ArmResource)
- `location` - Azure region (inherited from ArmResource)
- `tags` - Resource tags (inherited from ArmResource)
- `additionalVmPatch` - Additional VM patching configuration
- `enableAutomaticUpgrade` - Whether automatic upgrades are enabled
- `leastPrivilegeMode` - Least privilege mode setting
- `osType` - Operating system type
- `sqlImageOffer` - SQL Server image offer
- `sqlImageSku` - SQL Server image SKU
- `sqlServerLicenseType` - SQL Server license type
- `sqlManagement` - SQL management mode

## Relationships

- `VirtualMachine` → `HAS_SQL_VM` → `SqlVirtualMachine` - The virtual machine that hosts this SQL configuration
