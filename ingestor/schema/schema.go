package schema

import (
	"github.com/goccy/go-json"
	log "github.com/sirupsen/logrus"
	"golang.org/x/exp/slices"
)

type RelationshipType string
type NodeLabel string

type Node interface {
	MakeRelationships() []Relationship
}

type Relationship struct {
	SourceNodeID     string                 `json:"sourceNodeId"`
	TargetNodeID     string                 `json:"targetNodeId"`
	SourceLabel      NodeLabel              `json:"sourceLabel"`
	TargetLabel      NodeLabel              `json:"targetLabel"`
	RelationshipType RelationshipType       `json:"relationshipType"`
	Properties       map[string]interface{} `json:"properties"`
	SourceProperty   string                 `json:"sourceProperty"`
	TargetProperty   string                 `json:"targetProperty"`
}

const (
	// Relationships
	AssociatedTo  RelationshipType = "AssociatedTo"
	AttachedTo    RelationshipType = "AttachedTo"
	Authenticates RelationshipType = "Authenticates"
	ConnectedTo   RelationshipType = "ConnectedTo"
	Contains      RelationshipType = "Contains"
	Exposes       RelationshipType = "Exposes"
	HasAccess     RelationshipType = "HasAccess"
	HasConfig     RelationshipType = "HasConfig"
	HasDisk       RelationshipType = "HasDisk"
	HasIdentity   RelationshipType = "HasIdentity"
	HasInstance   RelationshipType = "HasInstance"
	HasRbac       RelationshipType = "HasRbac"
	HasRole       RelationshipType = "HasRole"
	Manages       RelationshipType = "Manages"
	MemberOf      RelationshipType = "MemberOf"
	Owns          RelationshipType = "Owns"
	Represents    RelationshipType = "Represents"
	RepresentedBy RelationshipType = "RepresentedBy"
	Trusts        RelationshipType = "Trusts"
)

const (
	// Node labels
	GraphObject           NodeLabel = "GraphObject"
	GraphApplication      NodeLabel = "GraphApplication"
	GraphDevice           NodeLabel = "GraphDevice"
	GraphGroup            NodeLabel = "GraphGroup"
	GraphRole             NodeLabel = "GraphRole"
	GraphServicePrincipal NodeLabel = "GraphServicePrincipal"
	GraphUser             NodeLabel = "GraphUser"

	ArmResource          NodeLabel = "ArmResource"
	AutomationAccount    NodeLabel = "AutomationAccount"
	AzureRbac            NodeLabel = "AzureRbac"
	Disk                 NodeLabel = "Disk"
	IpConfiguration      NodeLabel = "IpConfiguration"
	KeyVault             NodeLabel = "KeyVault"
	ManagedIdentity      NodeLabel = "ManagedIdentity"
	NetworkInterface     NodeLabel = "NetworkInterface"
	NetworkSecurityGroup NodeLabel = "NetworkSecurityGroup"
	PublicIPAddress      NodeLabel = "PublicIPAddress"
	ResourceGroup        NodeLabel = "ResourceGroup"
	Runbook              NodeLabel = "Runbook"
	SSHPublicKey         NodeLabel = "SSHPublicKey"
	StorageAccount       NodeLabel = "StorageAccount"
	Subnet               NodeLabel = "Subnet"
	Subscription         NodeLabel = "Subscription"
	Tenant               NodeLabel = "Tenant"
	VirtualNetwork       NodeLabel = "VirtualNetwork"
	VirtualMachine       NodeLabel = "VirtualMachine"
)

func AsNeo4j(node *Node) map[string]interface{} {

	objectMap, err := json.Marshal(node)
	if err != nil {
		return nil
	}

	var objectMapInterface map[string]interface{}
	json.Unmarshal(objectMap, &objectMapInterface)

	// We don't want to include these fields in the map
	fieldsToExclude := []string{"members", "owners", "appRoles", "registeredUsers", "properties", "identity"}
	for _, field := range fieldsToExclude {
		delete(objectMapInterface, field)
	}

	// We need to convert flatten maps to an array
	// We'll want to keep order of the keys for things like extensionAttributes
	for key, value := range objectMapInterface {

		// Skip properties
		if key == "properties" {
			continue
		}
		_, isMap := value.(map[string]interface{})
		if isMap {

			var valueArray []any
			var keys []string

			for k := range value.(map[string]interface{}) {
				keys = append(keys, k)
			}
			slices.Sort(keys)

			// Check the type of each value in the map and convert it to the right type
			for _, k := range keys {
				var valueData = value.(map[string]interface{})[k]
				var convertedValue interface{}

				switch kType := valueData.(type) {
				case string:
					convertedValue = valueData.(string)
				case bool:
					convertedValue = valueData.(bool)
				case float64:
					convertedValue = valueData.(float64)
				case int:
					convertedValue = valueData.(int)
				case []interface{}:
					convertedValue = valueData.([]interface{})
				case nil:
					convertedValue = nil
				default:
					log.Warnf("Unknown type %T for key %s: \n %s", kType, k, objectMapInterface)
				}
				if convertedValue != nil {
					valueArray = append(valueArray, k, convertedValue)
				}
			}
			objectMapInterface[key] = valueArray
		}
	}
	return objectMapInterface
}
