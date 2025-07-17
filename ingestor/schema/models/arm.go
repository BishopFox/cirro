package models

import (
	"errors"
	"reflect"

	"github.com/bishopfox/cirro/ingestor/schema"
)

// Resources may have a managed identity
type ManagedIdentity struct {
	Type        string `json:"type"`
	PrincipalID string `json:"principalId"`
	TenantID    string `json:"tenantId"`
}

// ArmResource is the base resource type
type ArmResource struct {
	ID         string                 `json:"id"`
	Identity   ManagedIdentity        `json:"identity"`
	Kind       string                 `json:"kind"`
	Location   string                 `json:"location"`
	Name       string                 `json:"name"`
	Properties map[string]interface{} `json:"properties"`
	Type       string                 `json:"type"`
	Tags       map[string]interface{} `json:"tags"`
}

type AzureRbac struct {
	ID          string                 `json:"id"`
	Description string                 `json:"description"`
	Name        string                 `json:"name"`
	Properties  map[string]interface{} `json:"properties"`
	RoleName    string                 `json:"roleName"`
	RoleType    string                 `json:"roleType"`
}

func (m *ManagedIdentity) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship
	return relationships
}

func GetNodeManagedIdentity(node schema.Node) (*ManagedIdentity, error) {
	val := reflect.ValueOf(node).Elem()
	if val.Kind() == reflect.Struct {
		field := val.FieldByName("ArmResource")
		if field.IsValid() {
			// Ensure the field is exported and can be interfaced
			if field.CanInterface() {
				armResource, ok := field.Interface().(ArmResource)
				if ok {
					// Do something with armResource
					return &armResource.Identity, nil
				}
			}
		}
	}
	return nil, errors.New("")
}

func GetNodeIDProperty(node schema.Node) string {
	val := reflect.ValueOf(node).Elem()
	if val.Kind() == reflect.Struct {
		field := val.FieldByName("ArmResource")
		if field.IsValid() {
			// Ensure the field is exported and can be interfaced
			if field.CanInterface() {
				armResource, ok := field.Interface().(ArmResource)
				if ok {
					// Do something with armResource
					return armResource.ID
				}
			}
		}
	}
	return ""
}
