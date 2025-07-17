package collectors

import (
	"fmt"
)

type Cloud int

const (
	Public Cloud = iota
	China
	German
	UsGov
	Invalid
)

type CloudEndpoints struct {
	// Cloud endpoints
	MsGraphUrl    string
	AadGraphUrl   string
	ArmUrl        string
	VaultUrl      string
	TokenEndpoint string
}

func ParseCloudFromString(s string) (Cloud, error) {
	switch s {
	case "china":
		return China, nil
	case "german":
		return German, nil
	case "usgov":
		return UsGov, nil
	case "public":
		return Public, nil
	default:
		return Invalid, fmt.Errorf("invalid Cloud enum %s", s)
	}
}

// Return the CloudEndpoints for the cloud
func (c Cloud) GetEndpoints() CloudEndpoints {
	switch c {
	case Public:
		return CloudEndpoints{
			MsGraphUrl:    "https://graph.microsoft.com",
			AadGraphUrl:   "https://graph.windows.net",
			ArmUrl:        "https://management.azure.com",
			VaultUrl:      "https://vault.azure.net",
			TokenEndpoint: "https://login.microsoftonline.com",
		}
	case China:
		return CloudEndpoints{
			MsGraphUrl:    "https://microsoftgraph.chinacloudapi.cn",
			AadGraphUrl:   "https://graph.chinacloudapi.cn",
			ArmUrl:        "https://management.chinacloudapi.cn",
			VaultUrl:      "https://vault.azure.cn",
			TokenEndpoint: "https://login.chinacloudapi.cn",
		}
	case German:
		return CloudEndpoints{
			MsGraphUrl:    "https://graph.microsoft.de",
			AadGraphUrl:   "https://graph.cloudapi.de",
			ArmUrl:        "https://management.microsoftazure.de",
			VaultUrl:      "https://vault.microsoftazure.de",
			TokenEndpoint: "https://login.microsoftonline.de",
		}
	case UsGov:
		return CloudEndpoints{
			MsGraphUrl:    "https://graph.microsoft.us",
			AadGraphUrl:   "https://graph.microsoftazure.us",
			ArmUrl:        "https://management.usgovcloudapi.net",
			VaultUrl:      "https://vault.usgovcloudapi.net",
			TokenEndpoint: "https://login.microsoftonline.us",
		}
	default:
		return CloudEndpoints{
			MsGraphUrl:    "https://graph.microsoft.com",
			AadGraphUrl:   "https://graph.windows.net",
			ArmUrl:        "https://management.azure.com",
			VaultUrl:      "https://vault.azure.net",
			TokenEndpoint: "https://login.microsoftonline.com",
		}
	}
}
