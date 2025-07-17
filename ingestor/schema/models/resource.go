package models

import (
	"strings"

	"github.com/bishopfox/cirro/ingestor/schema"
)

type Tenant struct {
	ID             string   `json:"id"`
	DisplayName    string   `json:"displayName"`
	TenantID       string   `json:"tenantId"`
	TenantCategory string   `json:"tenantCategory"`
	Country        string   `json:"country"`
	Domains        []string `json:"domains"`
	DefaultDomain  string   `json:"defaultDomain"`
	TenantType     string   `json:"tenantType"`
}

type ManagementGroup struct {
	ID         string                 `json:"id"`
	Name       string                 `json:"name"`
	Properties map[string]interface{} `json:"properties"`
	Type       string                 `json:"type"`
}

type ResourceGroup struct {
	ID         string                 `json:"id"`
	Location   string                 `json:"location"`
	Name       string                 `json:"name"`
	Properties map[string]interface{} `json:"properties"`
	Tags       map[string]interface{} `json:"tags"`
	Type       string                 `json:"type"`
}

type Subscription struct {
	ID               string   `json:"id"`
	DisplayName      string   `json:"displayName"`
	SubscriptionID   string   `json:"subscriptionId"`
	TenantID         string   `json:"tenantId"`
	State            string   `json:"state"`
	ManagedByTenants []string `json:"managedByTenants"`
}

func (a *ArmResource) MakeRelationships() []schema.Relationship {
	return []schema.Relationship{}
}

func (r *AzureRbac) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship

	relationships = append(relationships, schema.Relationship{
		SourceNodeID:     r.Properties["principalId"].(string),
		TargetNodeID:     r.Properties["scope"].(string),
		SourceLabel:      schema.GraphObject,
		TargetLabel:      schema.ArmResource,
		RelationshipType: schema.RelationshipType(strings.ReplaceAll(r.RoleName, " ", "")), // Relationship type is the role name without spaces
		Properties: map[string]interface{}{
			"roleName":    r.RoleName,
			"roleType":    r.RoleType,
			"description": r.Description,
		},
	})
	return relationships
}

func (r *ResourceGroup) MakeRelationships() []schema.Relationship {
	return []schema.Relationship{}
}

func (s *Subscription) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship

	relationships = append(relationships, schema.Relationship{
		SourceNodeID:     "/tenants/" + s.TenantID,
		TargetNodeID:     s.ID,
		SourceLabel:      schema.Tenant,
		TargetLabel:      schema.Subscription,
		RelationshipType: schema.Contains,
	})
	return relationships
}

func (t *Tenant) MakeRelationships() []schema.Relationship {
	return []schema.Relationship{}
}
