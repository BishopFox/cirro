package models

import "github.com/bishopfox/cirro/ingestor/schema"

type KeyVault struct {
	ArmResource
	PropertyFields []string
}

func (k *KeyVault) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship

	if k.Properties["accessPolicies"] != nil {
		for _, accessPolicy := range k.Properties["accessPolicies"].([]interface{}) {
			relationships = append(relationships, schema.Relationship{
				SourceNodeID:     accessPolicy.(map[string]interface{})["objectId"].(string),
				TargetNodeID:     k.ID,
				SourceLabel:      schema.GraphObject,
				TargetLabel:      schema.KeyVault,
				RelationshipType: schema.HasAccess,
			})
		}
	}
	return relationships
}
