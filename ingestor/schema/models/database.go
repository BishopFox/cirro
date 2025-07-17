package models

import "github.com/bishopfox/cirro/ingestor/schema"

type DatabaseAccount struct {
	ArmResource
	PropertyFields []string
}

type SqlServer struct {
	ArmResource
	PropertyFields []string
}

type SqlServerDatabase struct {
	ArmResource
	PropertyFields []string
}

func (d *DatabaseAccount) MakeRelationships() []schema.Relationship { return nil }

func (d *SqlServer) MakeRelationships() []schema.Relationship { return nil }

func (d *SqlServerDatabase) MakeRelationships() []schema.Relationship { return nil }
