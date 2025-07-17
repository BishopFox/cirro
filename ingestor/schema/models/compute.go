package models

import "github.com/bishopfox/cirro/ingestor/schema"

type Disk struct {
	ArmResource
	PropertyFields []string
	managedBy      string
}

type SSHPublicKey struct {
	ArmResource
	PropertyFields []string
}

type VirtualMachine struct {
	ArmResource
	PropertyFields []string
}

func (d *Disk) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship
	if d.managedBy != "" {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     d.managedBy,
			TargetNodeID:     d.ID,
			SourceLabel:      schema.VirtualMachine,
			TargetLabel:      schema.Disk,
			RelationshipType: schema.HasDisk,
		})
	}
	return relationships
}

func (v *VirtualMachine) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship
	for _, nic := range v.Properties["networkProfile"].(map[string]interface{})["networkInterfaces"].([]interface{}) {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     nic.(map[string]interface{})["id"].(string),
			TargetNodeID:     v.ID,
			SourceLabel:      schema.NetworkInterface,
			TargetLabel:      schema.VirtualMachine,
			RelationshipType: schema.AssociatedTo,
		})
	}

	osDisk := v.Properties["storageProfile"].(map[string]interface{})["osDisk"].(map[string]interface{})
	relationships = append(relationships, schema.Relationship{
		SourceNodeID:     v.ID,
		TargetNodeID:     osDisk["managedDisk"].(map[string]interface{})["id"].(string),
		SourceLabel:      schema.VirtualMachine,
		TargetLabel:      schema.Disk,
		RelationshipType: schema.HasDisk,
	})

	// for _, sshkey := range v.Properties["osProfile"].(map[string]interface{})["ssh"].(map[string]interface{})["publicKeys"].([]interface{}) {
	// 	relationships = append(relationships, schema.Relationship{
	// 		SourceNodeID:     v.ID,

	return relationships
}
