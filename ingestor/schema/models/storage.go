package models

import "github.com/bishopfox/cirro/ingestor/schema"

type StorageAccount struct {
	ArmResource
	PropertyFields []string
}

func (s *StorageAccount) MakeRelationships() []schema.Relationship { return nil }
