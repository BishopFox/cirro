package core

import (
	"context"

	"github.com/Jeffail/gabs/v2"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j"
	log "github.com/sirupsen/logrus"
)

func (i *CirroIngestor) ProcessTenants() error {

	var properties = []string{
		"id",
		"displayName",
		"tenantId",
		"countryCode",
		"domains",
		"defaultDomain",
		"tenantCategory",
		"tenantType",
	}

	const TenantCreateQuery = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:Tenant {id: row.id})
		SET obj += {
			id : row.id,
			displayName : row.displayName,
			tenantId : row.tenantId,
			countryCode : row.countryCode,
			domains : row.domains,
			defaultDomain : row.defaultDomain,
			tenantCategory : row.tenantCategory,
			tenantType : row.tenantType
		}
	
	`

	// Get count of rows in tenants
	countRow := i.DB.QueryRow("SELECT COUNT(*) FROM tenants")
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		return nil
	}
	log.Printf("Processing %d tenants", count)

	// Limit the number of rows to process at once
	limit := 5000
	offset := 0

	for {
		var tenantList []map[string]interface{}

		if offset*limit <= count {
			rows, err := i.DB.Query("SELECT json(data) FROM tenants LIMIT ? OFFSET ?", limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			for rows.Next() {
				var tenantData []byte
				if err := rows.Scan(&tenantData); err != nil {
					log.Errorf("Error scanning tenant data: %v", err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(tenantData)
				if err != nil {
					log.Errorf("Error parsing tenant data: %v", err)
					continue
				}

				// Extract the properties from the JSON
				nodeObject := gabs.New()
				for _, property := range properties {
					nodeObject.Set(jsonParsed.Path(property).Data(), property)
				}

				tenantList = append(tenantList, nodeObject.Data().(map[string]interface{}))
			}

			// Merge the tenants into the graph
			nodeParams := map[string]interface{}{
				"batch":  tenantList,
				"labels": []string{Tenant},
			}
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, TenantCreateQuery, nodeParams, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error creating tenants: %v", err)
			}
			offset++
		} else {
			break
		}
	}
	log.Info("Finished processing tenants")
	return nil
}

func (i *CirroIngestor) ProcessSubscriptions() error {

	var properties = []string{
		"id",
		"displayName",
		"authorizationSource",
		"state",
		"subscriptionId",
		"tenantId",
	}

	const SubscriptionCreateQuery = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:Subscription:ArmResource {id: row.id})
		SET obj += {
			id : row.id,
			displayName : row.displayName,
			authorizationSource : row.authorizationSource,
			state : row.state,
			subscriptionId : row.subscriptionId,
			tenantId : row.tenantId
		}
		WITH obj, row
			MERGE (t:Tenant {tenantId: row.tenantId})
			MERGE (t)-[:CONTAINS]->(obj)
	`

	// Get count of rows in subscriptions
	countRow := i.DB.QueryRow("SELECT COUNT(*) FROM subscriptions")
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}

	if count == 0 {
		return nil
	}
	log.Printf("Processing %d subscriptions", count)

	// Limit the number of rows to process at once
	limit := 5000
	offset := 0

	for {
		var subscriptionList []map[string]interface{}

		if offset*limit <= count {
			rows, err := i.DB.Query("SELECT json(data) FROM subscriptions LIMIT ? OFFSET ?", limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			for rows.Next() {
				var subscriptionData []byte
				if err := rows.Scan(&subscriptionData); err != nil {
					log.Errorf("Error scanning subscription data: %v", err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(subscriptionData)
				if err != nil {
					log.Errorf("Error parsing subscription data: %v", err)
					continue
				}

				// Extract the properties from the JSON
				nodeObject := gabs.New()
				for _, property := range properties {
					nodeObject.Set(jsonParsed.Path(property).Data(), property)
				}

				subscriptionList = append(subscriptionList, nodeObject.Data().(map[string]interface{}))
			}
			// Merge the subscriptions into the graph
			nodeParams := map[string]interface{}{
				"batch":  subscriptionList,
				"labels": []string{Subscription},
			}
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, SubscriptionCreateQuery, nodeParams, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error creating subscriptions: %v", err)
			}
			offset++
		} else {
			break
		}
	}
	log.Info("Finished processing subscriptions")
	return nil
}

func (i *CirroIngestor) ProcessResourceGroups() error {

	var properties = []string{
		"id",
		"name",
		"location",
		"type",
	}

	const ResourceGroupCreateQuery = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:ResourceGroup:ArmResource
		SET obj += {
			id : row.id,
			name : row.name,
			location : row.location,
			type : row.type
		}
		WITH obj, row
			MERGE (s:Subscription {subscriptionId: row.sub_id})
			MERGE (s)-[:CONTAINS]->(obj)
	`

	// Get count of rows in resource groups
	countRow := i.DB.QueryRow("SELECT COUNT(*) FROM resources WHERE lower(resource_type) = 'microsoft.resources/resourcegroups'")
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		return nil
	}
	log.Printf("Processing %d resource groups", count)

	// Limit the number of rows to process at once
	limit := 5000
	offset := 0

	for {
		var resourceGroupList []map[string]interface{}

		if offset*limit <= count {
			rows, err := i.DB.Query("SELECT sub_id, json(data) FROM resources WHERE lower(resource_type) = 'microsoft.resources/resourcegroups' LIMIT ? OFFSET ?", limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			for rows.Next() {
				var resourceData []byte
				var subID string

				if err := rows.Scan(&subID, &resourceData); err != nil {
					log.Errorf("Error scanning resource group data: %v", err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(resourceData)
				if err != nil {
					log.Errorf("Error parsing resource group data: %v", err)
					continue
				}

				// Extract the properties from the JSON
				nodeObject := gabs.New()
				for _, property := range properties {
					nodeObject.Set(jsonParsed.Path(property).Data(), property)
				}
				nodeObject.Set(subID, "sub_id")
				resourceGroupList = append(resourceGroupList, nodeObject.Data().(map[string]interface{}))
			}

			// Merge the resource groups into the graph
			nodeParams := map[string]interface{}{
				"batch":  resourceGroupList,
				"labels": []string{ResourceGroup},
			}
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, ResourceGroupCreateQuery, nodeParams, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error creating resource groups: %v", err)
			}
			offset++
		} else {
			break
		}
	}
	log.Info("Finished processing resource groups")
	return nil
}

func (i *CirroIngestor) ProcessAzureRbac() error {
	var properties = []string{
		"id",
		"description",
		"roleName",
		"roleType",
		"properties",
	}

	const AzureRbacCreateQuery = `/*cypher*/
	UNWIND $batch as row
	WITH row
		CREATE (p:GraphObject {id: row.properties.principalId})
	WITH p, row
		MATCH (s:ArmResource {id: row.properties.scope})
		CALL apoc.merge.relationship(p, row.roleName, {}, {}, s) YIELD rel
		SET rel += {
			id : row.id,
			description : row.description,
			roleName : row.roleName,
			roleType : row.roleType
		}
	`

	// Get count of rows in azure rbac
	countRow := i.DB.QueryRow("SELECT COUNT(*) FROM roleAssignments")
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		return nil
	}
	log.Printf("Processing %d RBAC assignments", count)

	// Limit the number of rows to process at once
	limit := 500
	offset := 0

	for {
		var azureRbacList []map[string]interface{}

		if offset*limit <= count {
			rows, err := i.DB.Query("SELECT json(data) FROM roleAssignments LIMIT ? OFFSET ?", limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			for rows.Next() {
				var azureRbacData []byte
				if err := rows.Scan(&azureRbacData); err != nil {
					log.Errorf("Error scanning azure rbac data: %v", err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(azureRbacData)
				if err != nil {
					log.Errorf("Error parsing azure rbac data: %v", err)
					continue
				}

				// Extract the properties from the JSON
				nodeObject := gabs.New()
				for _, property := range properties {
					nodeObject.Set(jsonParsed.Path(property).Data(), property)
				}
				azureRbacList = append(azureRbacList, nodeObject.Data().(map[string]interface{}))
			}

			// Merge the azure rbac into the graph
			nodeParams := map[string]interface{}{
				"batch": azureRbacList,
			}

			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, AzureRbacCreateQuery, nodeParams, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error creating azure rbac: %v", err)
			}
			offset++
		} else {
			break
		}
	}
	log.Info("Finished processing RBAC assignments")
	return nil
}

func (i *CirroIngestor) ProcessArmResources() error {
	var properties = []string{
		"id",
		"identity",
		"kind",
		"location",
		"name",
		"type",
		"tags",
	}

	const ResourceCreateQuery = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj += {
			id : row.id,
			kind : row.kind,
			location : row.location,
			name : row.name,
			type : row.type,
			tags : [key in keys(row.tags) | key + ":" + row.tags[key]]
		}
		WITH obj, row
			WHERE row.identity IS NOT NULL AND lower(row.identity.type) = 'systemassigned'
				CREATE (i:GraphObject {id: row.identity.principalId})
				CREATE (obj)-[:HAS_IDENTITY]->(i)
		WITH obj, row
			WHERE row.identity IS NOT NULL AND lower(row.identity.type) = 'userassigned'
				CREATE (i:GraphObject {id: row.identity.principalId})
		WITH obj, row
			MERGE (rg:ResourceGroup {id: row.rg_id})
			MERGE (rg)-[:CONTAINS]->(obj)
	`

	// Get count of rows in resources
	countRow := i.DB.QueryRow("SELECT COUNT(*) FROM resources WHERE lower(resource_type) != 'microsoft.resources/resourcegroups'")
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		return nil
	}
	log.Printf("Processing %d resources as generic", count)

	// Limit the number of rows to process at once
	limit := 5000
	offset := 0

	for {
		var resourceList []map[string]interface{}

		if offset*limit <= count {
			rows, err := i.DB.Query("SELECT rg_id, json(data) FROM resources WHERE lower(resource_type) != 'microsoft.resources/resourcegroups' LIMIT ? OFFSET ?", limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			for rows.Next() {
				var resourceData []byte
				var rgID string

				if err := rows.Scan(&rgID, &resourceData); err != nil {
					log.Errorf("Error scanning resource data: %v", err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(resourceData)
				if err != nil {
					log.Errorf("Error parsing resource data: %v", err)
					continue
				}

				// Extract the properties from the JSON
				nodeObject := gabs.New()
				for _, property := range properties {
					nodeObject.Set(jsonParsed.Path(property).Data(), property)
				}

				nodeObject.Set(rgID, "rg_id")
				resourceList = append(resourceList, nodeObject.Data().(map[string]interface{}))
			}

			// Merge the resources into the graph
			nodeParams := map[string]interface{}{
				"batch":  resourceList,
				"labels": []string{ArmResource},
			}
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, ResourceCreateQuery, nodeParams, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error creating resources: %v", err)
			}
			offset++
		} else {
			break
		}
	}
	log.Info("Finished processing resources as generic")
	return nil
}

func (i *CirroIngestor) ProcessSpecificArmResource(cypherQuery string, properties []string, resourceType string) error {

	countRow := i.DB.QueryRow(`SELECT COUNT(*) FROM resources WHERE lower(resource_type) = ?`, resourceType)
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		return nil
	}
	log.Infof("Processing %d %s", count, resourceType)

	// Set limit and offset
	limit := 5000
	offset := 0

	for {
		var resourceList []map[string]interface{}

		if offset*limit <= count {
			rows, err := i.DB.Query(`SELECT rg_id, json(data) FROM resources WHERE lower(resource_type) = ? LIMIT ? OFFSET ?`, resourceType, limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			for rows.Next() {
				var rgID string
				var data []byte

				if err := rows.Scan(&rgID, &data); err != nil {
					log.Errorf("Error scanning %s: %s", resourceType, err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(data)
				if err != nil {
					log.Errorf("Error parsing %s: %s", resourceType, err)
					continue
				}

				// Extract the properties
				nodeObject := gabs.New()
				for _, property := range properties {
					nodeObject.Set(jsonParsed.Path(property).Data(), property)
				}

				// Set the resource group ID
				nodeObject.Set(rgID, "rg_id")
				resourceList = append(resourceList, nodeObject.Data().(map[string]interface{}))
			}

			nodeParams := map[string]interface{}{
				"batch": resourceList,
			}
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, cypherQuery, nodeParams, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error creating %s: %s", resourceType, err)
			}
			offset++
		} else {
			break
		}
	}
	return nil
}
