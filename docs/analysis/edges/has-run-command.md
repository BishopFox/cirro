# HAS_RUN_COMMAND

Links virtual machines to their associated run commands.

**Relationship:** `:HAS_RUN_COMMAND`

**Direction:** `VirtualMachine` → `VMRunCommand`

## Description

This relationship connects virtual machines to run commands that can be executed on them. Run commands allow for remote execution of scripts and commands on Azure virtual machines.

## Use Cases

- Identifying which run commands are configured on specific virtual machines
- Understanding remote execution capabilities and scripts associated with VMs
- Security analysis of commands that can be executed on virtual machines