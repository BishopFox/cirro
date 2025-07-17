package models

import (
	"github.com/bishopfox/cirro/ingestor/schema"
)

var NodeLabelToNodeMap = map[schema.NodeLabel]schema.Node{
	schema.GraphApplication:      &GraphApplication{},
	schema.GraphDevice:           &GraphDevice{},
	schema.GraphGroup:            &GraphGroup{},
	schema.GraphRole:             &GraphRole{},
	schema.GraphServicePrincipal: &GraphServicePrincipal{},
	schema.GraphUser:             &GraphUser{},

	schema.ArmResource:       &ArmResource{},
	schema.AutomationAccount: &AutomationAccount{},
	schema.AzureRbac:         &AzureRbac{},

	schema.Disk: &Disk{
		PropertyFields: []string{"osType",
			"hyperVGeneration",
			"diskSizeGB",
			"diskState",
			"timeCreated",
		},
	},

	schema.IpConfiguration: &IpConfiguration{
		PropertyFields: []string{"privateIPAddress",
			"privatedIPAllocationMethod",
			"primary",
			"privateIPAddressVersion",
		},
	},

	schema.KeyVault: &KeyVault{
		PropertyFields: []string{"enableSoftDelete",
			"softDeleteRetentionInDays",
			"enableRbacAuthorization",
			"enableSoftDelete",
			"enablePurgeProtection",
			"softDeleteRetentionInDays",
			"vaultUri",
		},
	},
	schema.ManagedIdentity: &ManagedIdentity{},

	schema.NetworkInterface: &NetworkInterface{
		PropertyFields: []string{"macAddress",
			"enableIpForwarding",
			"primary",
			"dnsSettings.dnsServers",
			"dnsSettings.appliedDnsServers",
			"dnsSettings.internalDomainNameSuffix",
			"nicType",
		},
	},

	schema.NetworkSecurityGroup: &NetworkSecurityGroup{},

	schema.PublicIPAddress: &PublicIPAddress{
		PropertyFields: []string{"ipAddress",
			"dnsSettings.fqdn",
			"publicIPAddressVersion",
			"publicIPAllocationMethod",
			"idleTimeoutInMinutes",
			"ipTags",
		},
	},

	schema.ResourceGroup: &ResourceGroup{},
	schema.Runbook:       &Runbook{},

	schema.SSHPublicKey: &SSHPublicKey{
		PropertyFields: []string{"publicKey"},
	},

	schema.StorageAccount: &StorageAccount{
		PropertyFields: []string{"accessTier",
			"allowBlobPublicAccess",
			"creationTime",
			"networkAcls.bypass",
			"networkAcls.defaultAction",
			"supportsHttpsTrafficOnly",
		},
	},

	schema.Subnet: &Subnet{
		PropertyFields: []string{"addressPrefix",
			"privateEndpointNetworkPolicies",
			"privateLinkServiceNetworkPolicies",
		},
	},

	schema.Subscription: &Subscription{},
	schema.Tenant:       &Tenant{},

	schema.VirtualNetwork: &VirtualNetwork{
		PropertyFields: []string{"enableDdosProtection",
			"addressSpace.addressPrefixes",
		},
	},
	schema.VirtualMachine: &VirtualMachine{
		PropertyFields: []string{"osProfile.computerName",
			"osProfile.adminUsername",
			"osProfile.allowExtensionOperations",
			"hardwareProfile.vmSize",
			"storageProfile.imageReference.publisher",
			"storageProfile.imageReference.offer",
			"storageProfile.imageReference.sku",
			"storageProfile.imageReference.exactVersion",
		},
	},
}

var ResourceTypeToNodeLabel = map[string]schema.NodeLabel{
	"microsoft.automation/automationaccounts":          schema.AutomationAccount,
	"microsoft.automation/automationaccounts/runbooks": schema.Runbook,
	"microsoft.computer/sshpublickeys":                 schema.SSHPublicKey,
	"microsoft.resources/resourcegroups":               schema.ResourceGroup,
	"microsoft.storage/storageaccounts":                schema.StorageAccount,
}
