package models

import "github.com/bishopfox/cirro/ingestor/schema"

type AutomationAccount struct {
	ArmResource
	PropertyFields []string
}

type Runbook struct {
	ArmResource
	PropertyFields []string
}

func (a *AutomationAccount) MakeRelationships() []schema.Relationship { return nil }

func (r *Runbook) MakeRelationships() []schema.Relationship { return nil }
