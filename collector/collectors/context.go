package collectors

import (
	"fmt"
	"time"

	"github.com/bishopfox/cirro/collector/credentials"
)

type AuthMethod int
type EnumMode int

const (
	//Authentication methods
	AccessToken AuthMethod = iota
	AzCliAuth
	ClientSecret
	ClientCert
)

// Enum modes
const (
	Graph EnumMode = iota
	ArmEnum
	BothEnum
	InvalidEnum
)

type ContextOptions struct {
	EnrichVaultCerts                bool
	KeyvaultElevate                 bool
	EnrichConditionalAccessPolicies bool
}

type CollectorContext struct {
	AuthMethod      AuthMethod
	EnumMode        EnumMode
	TenantID        string
	ClientId        string
	ClientSecret    string
	CertificatePath string
	ContextOptions  *ContextOptions
	Cloud           Cloud
	CloudEndpoints  CloudEndpoints
	MSGraphCred     credentials.AuthCredential
	ARMCred         credentials.AuthCredential
	VaultCred       credentials.AuthCredential
	AadGraphCred    credentials.AuthCredential
	RawToken        string
	DBPath          string
	OutputDB        *CirroDB
	GraphCounterMap map[GraphObjectType]int
	ArmCounterMap   map[string]int
	GraphSemaphore  chan bool
	ArmSemaphore    chan bool
}

func (a AuthMethod) String() string {
	return []string{"AccessToken", "AzCli", "ClientSecret", "ClientCert"}[a]
}

func (a AuthMethod) LongString() string {
	return []string{"Access Token", "Azure CLI", "Client Secret", "Client Certificate"}[a]
}

func (c EnumMode) String() string {
	return []string{"graph", "arm", "both"}[c]
}

func (c EnumMode) LongString() string {
	return []string{"Microsoft Graph", "Azure Resource Manager", "Graph/ARM"}[c]
}

func ParseEnumModeFromString(s string) (EnumMode, error) {
	switch s {
	case "graph":
		return Graph, nil
	case "arm":
		return ArmEnum, nil
	case "both":
		return BothEnum, nil
	default:
		return InvalidEnum, fmt.Errorf("invalid enum: %s", s)
	}
}

func DefaultCollectorContext() CollectorContext {
	fileTimestamp := time.Now().UTC().Format("20060102T150405Z")
	outputPath := "results-" + fileTimestamp

	return CollectorContext{
		ContextOptions:  &ContextOptions{},
		Cloud:           Public,
		EnumMode:        BothEnum,
		TenantID:        "",
		DBPath:          outputPath,
		GraphCounterMap: make(map[GraphObjectType]int),
		ArmCounterMap:   make(map[string]int),
		GraphSemaphore:  make(chan bool, 50),
		ArmSemaphore:    make(chan bool, 50),
	}
}
