package models

import "github.com/bishopfox/cirro/ingestor/schema"

type IpConfiguration struct {
	ArmResource
	PropertyFields []string
}

type NetworkInterface struct {
	ArmResource
	PropertyFields []string
}

type NetworkSecurityGroup struct {
	ArmResource
	PropertyFields []string
}

type PublicIPAddress struct {
	ArmResource
	PropertyFields []string
}

type Subnet struct {
	ArmResource
	PropertyFields []string
}

type VirtualNetwork struct {
	ArmResource
	PropertyFields []string
}

func (i *IpConfiguration) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship
	if publicIp := i.Properties["publicIPAddress"]; publicIp != nil {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     i.ID,
			TargetNodeID:     publicIp.(string),
			SourceLabel:      schema.ArmResource,
			TargetLabel:      schema.PublicIPAddress,
			RelationshipType: schema.Exposes,
		})
	}
	return relationships
}

func (n *NetworkInterface) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship
	return relationships
}

func (n *NetworkSecurityGroup) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship
	for _, intfc := range n.Properties["networkInterfaces"].([]interface{}) {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     intfc.(map[string]interface{})["id"].(string),
			TargetNodeID:     n.ID,
			SourceLabel:      schema.NetworkInterface,
			TargetLabel:      schema.NetworkSecurityGroup,
			RelationshipType: schema.AssociatedTo,
		})
	}
	return relationships
}

func (p *PublicIPAddress) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship
	if ipConfig := p.Properties["ipConfiguration"]; ipConfig != nil {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     ipConfig.(string),
			TargetNodeID:     p.ID,
			SourceLabel:      schema.IpConfiguration,
			TargetLabel:      schema.PublicIPAddress,
			RelationshipType: schema.Exposes,
		})
	}
	return relationships
}

func (s *Subnet) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship
	if nsg := s.Properties["networkSecurityGroup"]; nsg != nil {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     nsg.(string),
			TargetNodeID:     s.ID,
			SourceLabel:      schema.NetworkSecurityGroup,
			TargetLabel:      schema.Subnet,
			RelationshipType: schema.Contains,
		})
	}
	for _, ipConfig := range s.Properties["ipConfigurations"].([]interface{}) {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     s.ID,
			TargetNodeID:     ipConfig.(map[string]interface{})["id"].(string),
			SourceLabel:      schema.Subnet,
			TargetLabel:      schema.IpConfiguration,
			RelationshipType: schema.HasConfig,
		})
	}
	return relationships
}

func (v *VirtualNetwork) MakeRelationships() []schema.Relationship {
	var relationships []schema.Relationship
	for _, subnet := range v.Properties["subnets"].([]interface{}) {
		relationships = append(relationships, schema.Relationship{
			SourceNodeID:     v.ID,
			TargetNodeID:     subnet.(map[string]interface{})["id"].(string),
			SourceLabel:      schema.VirtualNetwork,
			TargetLabel:      schema.Subnet,
			RelationshipType: schema.Contains,
		})
	}
	return relationships
}
