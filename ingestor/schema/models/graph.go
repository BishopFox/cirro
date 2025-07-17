package models

import (
	"github.com/bishopfox/cirro/ingestor/schema"
)

type GraphObject struct {
	DisplayName string `json:"displayName"`
	ID          string `json:"id"`
}

type GraphApplication struct {
	GraphObject
	AppID                  string         `json:"appId"`
	AppRoleAssignments     []GraphAppRole `json:"appRoles"`
	AppOwnerOrganizationID string         `json:"appOwnerOrganizationId"`
	// KeyCredentials         []map[string]string `json:"keyCredentials"`
	Owners []string `json:"owners"`
	// PasswordCredentials    []map[string]string `json:"passwordCredentials"`
	PublisherName  string `json:"publisherName"`
	SignInAudience string `json:"signInAudience"`
}

type GraphAppRole struct {
	AllowedMemberTypes []string `json:"allowedMemberTypes"`
	Description        string   `json:"description"`
	DisplayName        string   `json:"displayName"`
	ID                 string   `json:"id"`
	IsEnabled          bool     `json:"isEnabled"`
	Value              string   `json:"value"`
}

type GraphDevice struct {
	GraphObject
	AccountEnabled                bool              `json:"accountEnabled"`
	DeviceID                      string            `json:"deviceId"`
	IsCompliant                   bool              `json:"isCompliant"`
	IsManaged                     bool              `json:"isManaged"`
	Manufacturer                  string            `json:"manufacturer"`
	Model                         string            `json:"model"`
	OnPremisesExtensionAttributes map[string]string `json:"onPremisesExtensionAttributes"`
	OnPremisesLastSyncDateTime    string            `json:"onPremisesLastSyncDateTime"`
	OnPremisesSyncEnabled         bool              `json:"onPremisesSyncEnabled"`
	OperatingSystem               string            `json:"operatingSystem"`
	OperatingSystemVersion        string            `json:"operatingSystemVersion"`
	ProfileType                   string            `json:"profileType"`
	RegisteredUsers               []string          `json:"registeredUsers"`
	TrustType                     string            `json:"trustType"`
}

type GraphServicePrincipal struct {
	GraphObject
	AccountEnabled         bool           `json:"accountEnabled"`
	AlternativeNames       []string       `json:"alternativeNames"`
	AppDisplayName         string         `json:"appDisplayName"`
	AppID                  string         `json:"appId"`
	AppOwnerOrganizationID string         `json:"appOwnerOrganizationId"`
	AppRoles               []GraphAppRole `json:"appRoles"`
	// KeyCredentials         []map[string]string `json:"keyCredentials"`
	Owners []string `json:"owners"`
	// PasswordCredentials    []map[string]string `json:"passwordCredentials"`
	PublisherName        string `json:"publisherName"`
	ServicePrincipalType string `json:"servicePrincipalType"`
}

type GraphGroup struct {
	GraphObject
	AppRoleAssignments           []GraphAppRole `json:"appRoles"`
	GroupTypes                   []string       `json:"groupTypes"`
	Members                      []string       `json:"members"`
	MembershipRule               string         `json:"membershipRule"`
	OnPremisesSecurityIdentifier string         `json:"onPremisesSecurityIdentifier"`
	OrganizationID               string         `json:"organizationId"`
	Owners                       []string       `json:"owners"`
	SecurityEnabled              bool           `json:"securityEnabled"`
	Visibility                   string         `json:"visibility"`
}

type GraphRole struct {
	GraphObject
	Description    string   `json:"description"`
	RoleTemplateID string   `json:"roleTemplateId"`
	Members        []string `json:"members"`
}

type GraphUser struct {
	GraphObject
	AccountEnabled                 bool              `json:"accountEnabled"`
	AppRoleAssignments             []GraphAppRole    `json:"appRoles"`
	CreationType                   string            `json:"creationType"`
	Department                     string            `json:"department"`
	EmployeeID                     string            `json:"employeeId"`
	JobTitle                       string            `json:"jobTitle"`
	Mail                           string            `json:"mail"`
	MailNickname                   string            `json:"mailNickname"`
	OnPremisesDistinguishedName    string            `json:"onPremisesDistinguishedName"`
	OnPremisesDomainName           string            `json:"onPremisesDomainName"`
	OnPremisesExtensionAttributes  map[string]string `json:"onPremisesExtensionAttributes"`
	OnPremisesSamAccountName       string            `json:"onPremisesSamAccountName"`
	OnPremisesSecurityIdentifier   string            `json:"onPremisesSecurityIdentifier"`
	OnPremisesUserPrincipalName    string            `json:"onPremisesUserPrincipalName"`
	RefreshTokensValidFromDateTime string            `json:"refreshTokensValidFromDateTime"`
	UserPrincipalName              string            `json:"userPrincipalName"`
	UserType                       string            `json:"userType"`
}

func (object GraphObject) MakeRelationships() []schema.Relationship { return nil }

func (app GraphApplication) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship

	// Add relationships to owners
	for _, owner := range app.Owners {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     owner,
			TargetNodeID:     app.ID,
			SourceLabel:      schema.GraphObject,
			TargetLabel:      schema.GraphApplication,
			RelationshipType: schema.Owns,
		})
	}
	return relationships
}

func (device GraphDevice) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship

	// Add relationships to registered users
	for _, user := range device.RegisteredUsers {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     user,
			TargetNodeID:     device.ID,
			SourceLabel:      schema.GraphObject,
			TargetLabel:      schema.GraphDevice,
			RelationshipType: schema.Owns,
		})
	}
	return relationships
}

func (sp GraphServicePrincipal) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship

	// If the service principal is a managed identity, add a relationship to the resource it is assigned to
	// Otherwise, add a relationship to the application it is associated with
	if sp.ServicePrincipalType == "ManagedIdentity" {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     sp.AlternativeNames[1], // The second alternative name is the resource ID
			TargetNodeID:     sp.ID,
			SourceLabel:      schema.ArmResource,
			TargetLabel:      schema.GraphServicePrincipal,
			RelationshipType: schema.HasIdentity,
		})
		// var isUserAssigned = strings.Contains(sp.AlternativeNames[1], "/providers/Microsoft.ManagedIdentity/userAssignedIdentities")
		// if isUserAssigned {
		// 	relationships = append(relationships, schema.Relationship{
		// 		SourceNodeID:     sp.AlternativeNames[1], // The second alternative name is the resource ID
		// 		TargetNodeID:     sp.ID,
		// 		SourceLabel:      schema.ArmResource,
		// 		TargetLabel:      schema.GraphServicePrincipal,
		// 		RelationshipType: schema.RepresentedBy,
		// 	})
		// } else {
		// 	relationships = append(relationships, schema.Relationship{
		// 		SourceNodeID:     sp.AlternativeNames[1], // The second alternative name is the resource ID
		// 		TargetNodeID:     sp.ID,
		// 		SourceLabel:      schema.ArmResource,
		// 		TargetLabel:      schema.GraphServicePrincipal,
		// 		RelationshipType: schema.HasIdentity,
		// 	})
		// }
	} else {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     sp.AppID,
			TargetNodeID:     sp.ID,
			SourceLabel:      schema.GraphApplication,
			TargetLabel:      schema.GraphServicePrincipal,
			RelationshipType: schema.HasInstance,
			SourceProperty:   "appId",
			// TargetProperty:   "appId",
		})
	}
	// Add relationships to owners
	for _, owner := range sp.Owners {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     owner,
			TargetNodeID:     sp.ID,
			SourceLabel:      schema.GraphObject,
			TargetLabel:      schema.GraphServicePrincipal,
			RelationshipType: schema.Owns,
		})
	}
	return relationships
}

func (group GraphGroup) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship

	// Add relationships to members
	for _, member := range group.Members {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     member,
			TargetNodeID:     group.ID,
			SourceLabel:      schema.GraphObject,
			TargetLabel:      schema.GraphGroup,
			RelationshipType: schema.MemberOf,
		})
	}

	// Add relationships to owners
	for _, owner := range group.Owners {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     owner,
			TargetNodeID:     group.ID,
			SourceLabel:      schema.GraphObject,
			TargetLabel:      schema.GraphGroup,
			RelationshipType: schema.Owns,
		})
	}
	return relationships
}

func (role GraphRole) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship

	// Add relationships to members
	for _, member := range role.Members {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     member,
			TargetNodeID:     role.ID,
			SourceLabel:      schema.GraphObject,
			TargetLabel:      schema.GraphRole,
			RelationshipType: schema.HasRole,
		})
	}
	return relationships
}
