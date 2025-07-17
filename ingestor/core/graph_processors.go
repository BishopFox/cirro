package core

import (
	"context"

	"github.com/Jeffail/gabs/v2"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j"
	log "github.com/sirupsen/logrus"
)

func (i *CirroIngestor) ProcessGraphApplications() error {
	// List of dotted notation strings to be used as properties
	var properties = []string{
		"displayName",
		"id",
		"appId",
		"appOwnerOrganizationId",
		"publisherName",
		"signInAudience",
		"keyCredentials",
		"owners",
	}

	const ApplicationCreateQuery = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:GraphApplication:GraphObject {id: row.id})
		SET obj += {
			displayName: row.displayName,
			id: row.id,
			appId: row.appId,
			appOwnerOrganizationId: row.appOwnerOrganizationId,
			publisherName: row.publisherName,
			signInAudience: row.signInAudience
		}
		FOREACH (owner IN row.owners |
			CREATE (o:GraphObject {id: owner.id})
			CREATE (o)-[:OWNS]->(obj)
		)
	WITH row, obj
		FOREACH (passwordCredential IN row.passwordCredentials |
			MERGE (password:ClientSecret {keyId: passwordCredential.keyId})
			SET password += {
				customKeyIdentifier: passwordCredential.customKeyIdentifier,
				keyId: passwordCredential.keyId,
				displayName: passwordCredential.displayName,
				startDateTime: passwordCredential.startDateTime,
				endDateTime: passwordCredential.endDateTime,
				hint: passwordCredential.hint,
				secretText: passwordCredential.secretText
			}
			MERGE (password)-[:AUTHENTICATES]->(obj)
		)
	WITH row, obj, [c IN row.keyCredentials WHERE c.customKeyIdentifier IS NOT NULL] AS keyCredentials
		FOREACH (keyCredential IN keyCredentials |
			MERGE (key:Certificate {thumbprint: keyCredential.customKeyIdentifier})
			SET key += {
				thumbprint: keyCredential.customKeyIdentifier,
				displayName: keyCredential.displayName,
				startDateTime: keyCredential.startDateTime,
				endDateTime: keyCredential.endDateTime,
				type: keyCredential.type,
				usage: keyCredential.usage
			}
			MERGE (key)-[:AUTHENTICATES]->(obj)
		)
	`

	// Get count of rows in applications
	countRow := i.DB.QueryRow(`SELECT COUNT(*) FROM applications`)
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		return nil
	}
	log.Infof("Processing %d applications", count)

	// Limit the number of rows to process in each batch to 5000
	limit := 5000
	offset := 0

	for {
		var applicationList []map[string]interface{}

		if offset*limit <= count {
			rows, err := i.DB.Query(`SELECT json(data) FROM applications LIMIT ? OFFSET ?`, limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			// Process each row
			// Convert the JSON data to gabs object
			// Then extract the properties from the object
			for rows.Next() {
				var applicationData []byte
				if err := rows.Scan(&applicationData); err != nil {
					log.Errorf("Error scanning row: %v", err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(applicationData)
				if err != nil {
					log.Errorf("Error parsing JSON: %v", err)
					continue
				}

				// Extract properties from the JSON object
				nodeObject := gabs.New()
				for _, property := range properties {
					value := jsonParsed.Path(property).Data()
					nodeObject.Set(value, property)
				}
				applicationList = append(applicationList, nodeObject.Data().(map[string]interface{}))

			}
			// Merge the nodes into the graph
			nodeParams := map[string]interface{}{
				"batch": applicationList,
			}
			log.Debugf("Merging %d application nodes from offset %d", len(applicationList), offset)
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, ApplicationCreateQuery, nodeParams, neo4j.EagerResultTransformer,
				neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error merging application nodes: %v", err)
			}
			offset++
		} else {
			break
		}
	}

	log.Info("Finished processing applications")

	return nil
}

func (i *CirroIngestor) ProcessGraphAdministrativeUnits() error {

	// List of dotted notation strings to be used as properties
	var properties = []string{
		"id",
		"description",
		"displayName",
		"isMemberManagementRestricted",
		"members",
		"membershipRule",
		"membershipRuleProcessingState",
		"membershipType",
		"scopedRoleMembers",
		"visibility",
	}

	const AUCreateQuery = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:AdministrativeUnit:GraphObject {id: row.id})
		SET obj += {
			description: row.description,
			displayName: row.displayName,
			isMemberManagementRestricted: row.isMemberManagementRestricted,
			membershipRule: row.membershipRule,
			membershipRuleProcessingState: row.membershipRuleProcessingState,
			membershipType: row.membershipType,
			visibility: row.visibility
		}

		FOREACH (memberObj IN row.members |
			CREATE (member:GraphObject {id: memberObj.id})
			CREATE (member)-[:MEMBER_OF]->(obj)
		)


	// We have to UNWIND here to match the roleId to an existing role with the same ID
	// FOREACH doesn't allow MATCH clauses inside
	WITH obj, coalesce(row.scopedRoleMembers, []) AS scopedRoleMembers
		UNWIND scopedRoleMembers AS memberObj
		CREATE (member:GraphObject {id: memberObj.roleMemberInfo_id})
		CREATE (member)-[r:HAS_ROLE]->(obj)
		SET r += {
			roleId: memberObj.roleId
		}

		WITH obj, memberObj, member, r
			OPTIONAL MATCH (role:GraphObject) WHERE role.id = memberObj.roleId
			WITH obj, memberObj, member, r, role
			WHERE role IS NOT NULL
			SET r += {
				roleName: role.displayName
			}
	`

	// Get count of rows in users
	countRow := i.DB.QueryRow(`SELECT COUNT(*) FROM administrativeUnits`)
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		return nil
	}
	log.Infof("Processing %d administrative units", count)

	// Limit the number of rows to process in each batch to 5000
	limit := 1
	offset := 0

	for {
		var auList []map[string]interface{}
		if offset*limit <= count {
			rows, err := i.DB.Query(`SELECT json(data) FROM administrativeUnits LIMIT ? OFFSET ?`, limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			// Process each row
			// Convert the JSON data to gabs object
			// Then extract the properties from the object
			for rows.Next() {
				var userData []byte
				if err := rows.Scan(&userData); err != nil {
					log.Errorf("Error scanning row: %v", err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(userData)
				if err != nil {
					log.Errorf("Error parsing JSON: %v", err)
					continue
				}

				// Extract properties from the JSON object
				nodeObject := gabs.New()
				for _, property := range properties {
					value := jsonParsed.Path(property).Data()
					nodeObject.Set(value, property)
				}
				auList = append(auList, nodeObject.Data().(map[string]interface{}))
			}
			// log.Info(auList)
			// Merge the nodes into the graph
			params := map[string]interface{}{
				"batch": auList,
			}
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, AUCreateQuery, params, neo4j.EagerResultTransformer,
				neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error merging administrative unit nodes: %v", err)
			}

			offset++
		} else {
			break
		}
	}

	log.Info("Finished processing administrative units")
	return nil
}

func (i *CirroIngestor) ProcessGraphDevices() error {
	// List of dotted notation strings to be used as properties
	var properties = []string{
		"displayName",
		"id",
		"accountEnabled",
		"deviceId",
		"isCompliant",
		"isManaged",
		"manufacturer",
		"model",
		"onPremisesLastSyncDateTime",
		"onPremisesSyncEnabled",
		"operatingSystem",
		"operatingSystemVersion",
		"profileType",
		"trustType",
		"registeredUsers",
	}

	const DeviceCreateQuery = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:GraphDevice:GraphObject {id: row.id})
		SET obj += {
			displayName: row.displayName,
			id: row.id,
			accountEnabled: row.accountEnabled,
			deviceId: row.deviceId,
			isCompliant: row.isCompliant,
			isManaged: row.isManaged,
			manufacturer: row.manufacturer,
			model: row.model,
			onPremisesLastSyncDateTime: row.onPremisesLastSyncDateTime,
			onPremisesSyncEnabled: row.onPremisesSyncEnabled,
			operatingSystem: row.operatingSystem,
			operatingSystemVersion: row.operatingSystemVersion,
			profileType: row.profileType,
			trustType: row.trustType
		}
	FOREACH (user IN row.registeredUsers |
		CREATE (owner:GraphObject {id: user.id})
		CREATE (owner)-[:OWNS]->(obj)
	)
	
	`

	// Get count of rows in devices
	countRow := i.DB.QueryRow(`SELECT COUNT(*) FROM devices`)
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		return nil
	}
	log.Infof("Processing %d devices", count)

	// Limit the number of rows to process in each batch to 5000
	limit := 5000
	offset := 0

	for {
		var deviceList []map[string]interface{}

		if offset*limit <= count {
			rows, err := i.DB.Query(`SELECT json(data) FROM devices LIMIT ? OFFSET ?`, limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			// Process each row
			// Convert the JSON data to gabs object
			// Then extract the properties from the object
			for rows.Next() {
				var deviceData []byte
				if err := rows.Scan(&deviceData); err != nil {
					log.Errorf("Error scanning row: %v", err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(deviceData)
				if err != nil {
					log.Errorf("Error parsing JSON: %v", err)
					continue
				}

				// Extract properties from the JSON object
				nodeObject := gabs.New()
				for _, property := range properties {
					value := jsonParsed.Path(property).Data()
					nodeObject.Set(value, property)
				}

				deviceList = append(deviceList, nodeObject.Data().(map[string]interface{}))
			}
			// Merge the nodes into the graph
			nodeParams := map[string]interface{}{
				"batch": deviceList,
			}
			log.Debugf("Merging %d device nodes from offset %d", len(deviceList), offset)
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, DeviceCreateQuery, nodeParams, neo4j.EagerResultTransformer,
				neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error merging device nodes: %v", err)
			}
			offset++
		} else {
			break
		}
	}
	log.Info("Finished processing devices")
	return nil
}

func (i *CirroIngestor) ProcessGraphGroups() error {
	// List of dotted notation strings to be used as properties
	var properties = []string{
		"displayName",
		"id",
		"groupTypes",
		"membershipRule",
		"membershipRuleProcessingState",
		"onPremisesSecurityIdentifier",
		"organizationId",
		"securityEnabled",
		"visibility",
		"owners",
		"members",
	}

	const GroupCreateQuery = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:GraphGroup:GraphObject {id: row.id})
		SET obj += {
			displayName: row.displayName,
			id: row.id,
			groupTypes: row.groupTypes,
			membershipRule: row.membershipRule,
			membershipRuleProcessingState: row.membershipRuleProcessingState,
			onPremisesSecurityIdentifier: row.onPremisesSecurityIdentifier,
			organizationId: row.organizationId,
			securityEnabled: row.securityEnabled,
			visibility: row.visibility
		}
	FOREACH (owner IN row.owners |
		CREATE (o:GraphObject {id: owner.id})
		CREATE (o)-[:OWNS]->(obj)
	)
	FOREACH (member IN row.members |
		CREATE (m:GraphObject {id: member.id})
		CREATE (m)-[:MEMBER_OF]->(obj)
	)
	
	`
	// Get count of rows in users
	countRow := i.DB.QueryRow(`SELECT COUNT(*) FROM groups`)
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		return nil
	}
	log.Infof("Processing %d groups", count)

	// Limit the number of rows to process in each batch to 5000
	limit := 1000
	offset := 0

	for {
		var groupList []map[string]interface{}

		if offset*limit <= count {
			rows, err := i.DB.Query(`SELECT json(data) FROM groups LIMIT ? OFFSET ?`, limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			// Process each row
			// Convert the JSON data to gabs object
			// Then extract the properties from the object
			for rows.Next() {
				var groupData []byte
				if err := rows.Scan(&groupData); err != nil {
					log.Errorf("Error scanning row: %v", err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(groupData)
				if err != nil {
					log.Errorf("Error parsing JSON: %v", err)
					continue
				}
				// Extract properties from the JSON object
				nodeObject := gabs.New()
				for _, property := range properties {
					value := jsonParsed.Path(property).Data()
					nodeObject.Set(value, property)
				}

				groupList = append(groupList, nodeObject.Data().(map[string]interface{}))
			}
			// Merge the nodes into the graph
			nodeParams := map[string]interface{}{
				"batch": groupList,
			}
			log.Debugf("Merging %d group nodes from offset %d", len(groupList), offset)
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, GroupCreateQuery, nodeParams, neo4j.EagerResultTransformer,
				neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error merging group nodes: %v", err)
			}
			offset++
		} else {
			break
		}
	}

	log.Info("Finished processing groups")
	return nil
}

func (i *CirroIngestor) ProcessGraphRoles() error {
	// List of dotted notation strings to be used as properties
	var properties = []string{
		"displayName",
		"id",
		"description",
		"roleTemplateId",
		"members",
	}

	const RoleCreateQuery = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:GraphRole:GraphObject {id: row.id})
		SET obj += {
			displayName: row.displayName,
			id: row.id,
			roleDescription: row.description,
			roleTemplateId: row.roleTemplateId
		}
	FOREACH (member IN row.members |
		CREATE (m:GraphObject {id: member.id})
		CREATE (m)-[:HAS_ROLE]->(obj)
	)
	
	`

	// Get count of rows in roles
	countRow := i.DB.QueryRow(`SELECT COUNT(*) FROM directoryRoles`)
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		return nil
	}
	log.Infof("Processing %d roles", count)

	// Limit the number of rows to process in each batch to 5000
	limit := 5000
	offset := 0

	for {
		var roleList []map[string]interface{}

		if offset*limit <= count {
			rows, err := i.DB.Query(`SELECT json(data) FROM directoryRoles LIMIT ? OFFSET ?`, limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			// Process each row
			// Convert the JSON data to gabs object
			// Then extract the properties from the object
			for rows.Next() {
				var roleData []byte
				if err := rows.Scan(&roleData); err != nil {
					log.Errorf("Error scanning row: %v", err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(roleData)
				if err != nil {
					log.Errorf("Error parsing JSON: %v", err)
					continue
				}
				// Extract properties from the JSON object
				nodeObject := gabs.New()
				for _, property := range properties {
					value := jsonParsed.Path(property).Data()
					nodeObject.Set(value, property)
				}

				roleList = append(roleList, nodeObject.Data().(map[string]interface{}))
			}
			// Merge the nodes into the graph
			nodeParams := map[string]interface{}{
				"batch": roleList,
			}
			log.Debugf("Merging %d role nodes from offset %d", len(roleList), offset)
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, RoleCreateQuery, nodeParams, neo4j.EagerResultTransformer,
				neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error merging role nodes: %v", err)
			}
			offset++
		} else {
			break
		}
	}
	log.Info("Finished processing roles")
	return nil
}

func (i *CirroIngestor) ProcessGraphServicePrincipals() error {

	// List of dotted notation strings to be used as properties
	var properties = []string{
		"displayName",
		"id",
		"accountEnabled",
		"alternativeNames",
		"appId",
		"appOwnerOrganizationId",
		"publisherName",
		"servicePrincipalType",
		"replyUrls",
		"servicePrincipalNames",
		"keyCredentials",
		"passwordCredentials",
		"owners",
	}

	const ServicePrincipalCreateQuery = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:GraphServicePrincipal:GraphObject {id: row.id})
		SET obj += {
			displayName: row.displayName,
			id: row.id,
			accountEnabled: row.accountEnabled,
			alternativeNames: row.alternativeNames,
			appId: row.appId,
			appOwnerOrganizationId: row.appOwnerOrganizationId,
			publisherName: row.publisherName,
			servicePrincipalType: row.servicePrincipalType,
			replyUrls: row.replyUrls,
			servicePrincipalNames: row.servicePrincipalNames
		}
		MERGE (app:GraphApplication {appId: row.appId})
		MERGE (app)-[:REPRESENTED_BY]->(obj)

		FOREACH (owner IN row.owners |
			CREATE (o:GraphObject {id: owner.id})
			CREATE (o)-[:OWNS]->(obj)
		)
	WITH row, obj
		FOREACH (passwordCredential IN row.passwordCredentials |
			MERGE (password:ClientSecret {keyId: passwordCredential.keyId})
			SET password += {
				customKeyIdentifier: passwordCredential.customKeyIdentifier,
				keyId: passwordCredential.keyId,
				displayName: passwordCredential.displayName,
				startDateTime: passwordCredential.startDateTime,
				endDateTime: passwordCredential.endDateTime,
				hint: passwordCredential.hint,
				secretText: passwordCredential.secretText
			}
			MERGE (password)-[:AUTHENTICATES]->(obj)
		)
	WITH row, obj, [c IN row.keyCredentials WHERE c.customKeyIdentifier IS NOT NULL] AS keyCredentials
		FOREACH (keyCredential IN keyCredentials |
			MERGE (key:Certificate {thumbprint: keyCredential.customKeyIdentifier})
			SET key += {
				thumbprint: keyCredential.customKeyIdentifier,
				displayName: keyCredential.displayName,
				startDateTime: keyCredential.startDateTime,
				endDateTime: keyCredential.endDateTime,
				type: keyCredential.type,
				usage: keyCredential.usage
			}
			MERGE (key)-[:AUTHENTICATES]->(obj)
		)
	`

	// Get count of rows in service principals
	countRow := i.DB.QueryRow(`SELECT COUNT(*) FROM servicePrincipals`)
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		return nil
	}
	log.Infof("Processing %d service principals", count)

	// Limit the number of rows to process in each batch to 5000
	limit := 1000
	offset := 0

	for {
		var servicePrincipalList []map[string]interface{}

		if offset*limit <= count {
			rows, err := i.DB.Query(`SELECT json(data) FROM servicePrincipals LIMIT ? OFFSET ?`, limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			// Process each row
			// Convert the JSON data to gabs object
			// Then extract the properties from the object
			for rows.Next() {
				var servicePrincipalData []byte
				if err := rows.Scan(&servicePrincipalData); err != nil {
					log.Errorf("Error scanning row: %v", err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(servicePrincipalData)
				if err != nil {
					log.Errorf("Error parsing JSON: %v", err)
					continue
				}
				// Extract properties from the JSON object
				nodeObject := gabs.New()
				for _, property := range properties {
					value := jsonParsed.Path(property).Data()
					nodeObject.Set(value, property)
				}

				servicePrincipalList = append(servicePrincipalList, nodeObject.Data().(map[string]interface{}))
			}
			// Merge the nodes into the graph
			nodeParams := map[string]interface{}{
				"batch": servicePrincipalList,
			}
			log.Debugf("Merging %d service principal nodes from offset %d", len(servicePrincipalList), offset)
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, ServicePrincipalCreateQuery, nodeParams, neo4j.EagerResultTransformer,
				neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error merging service principal nodes: %v", err)
			}
			offset++
		} else {
			break
		}
	}

	log.Info("Finished processing service principals")
	return nil
}

func (i *CirroIngestor) ProcessGraphUsers() error {

	// List of dotted notation strings to be used as properties
	var properties = []string{
		"accountEnabled",
		"city",
		"companyName",
		"country",
		"creationType",
		"department",
		"displayName",
		"givenName",
		"id",
		"jobTitle",
		"mail",
		"mailNickname",
		"mobilePhone",
		"officeLocation",
		"onPremisesDistinguishedName",
		"onPremisesSamAccountName",
		"onPremisesSecurityIdentifier",
		"onPremisesUserPrincipalName",
		"refreshTokensValidFromDateTime",
		"securityIdentifier",
		"state",
		"surname",
		"userPrincipalName",
		"userType",
	}

	const UserCreateQuery = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:GraphUser:GraphObject {id: row.id})
		SET obj += {
			accountEnabled: row.accountEnabled,
			city: row.city,
			companyName: row.companyName,
			country: row.country,
			creationType: row.creationType,
			department: row.department,
			displayName: row.displayName,
			givenName: row.givenName,
			jobTitle: row.jobTitle,
			mail: row.mail,
			mailNickname: row.mailNickname,
			mobilePhone: row.mobilePhone,
			officeLocation: row.officeLocation,
			onPremisesDistinguishedName: row.onPremisesDistinguishedName,
			onPremisesSamAccountName: row.onPremisesSamAccountName,
			onPremisesSecurityIdentifier: row.onPremisesSecurityIdentifier,
			onPremisesUserPrincipalName: row.onPremisesUserPrincipalName,
			refreshTokensValidFromDateTime: row.refreshTokensValidFromDateTime,
			securityIdentifier: row.securityIdentifier,
			state: row.state,
			surname: row.surname,
			userPrincipalName: row.userPrincipalName,
			userType: row.userType
		}
	
	`

	// Get count of rows in users
	countRow := i.DB.QueryRow(`SELECT COUNT(*) FROM users`)
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return err
	}
	if count == 0 {
		return nil
	}
	log.Infof("Processing %d users", count)

	// Limit the number of rows to process in each batch to 5000
	limit := 5000
	offset := 0

	for {
		var userList []map[string]interface{}
		if offset*limit <= count {
			rows, err := i.DB.Query(`SELECT json(data) FROM users LIMIT ? OFFSET ?`, limit, offset*limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			// Process each row
			// Convert the JSON data to gabs object
			// Then extract the properties from the object
			for rows.Next() {
				var userData []byte
				if err := rows.Scan(&userData); err != nil {
					log.Errorf("Error scanning row: %v", err)
					continue
				}
				jsonParsed, err := gabs.ParseJSON(userData)
				if err != nil {
					log.Errorf("Error parsing JSON: %v", err)
					continue
				}

				// Extract properties from the JSON object
				nodeObject := gabs.New()
				for _, property := range properties {
					value := jsonParsed.Path(property).Data()
					nodeObject.Set(value, property)
				}
				userList = append(userList, nodeObject.Data().(map[string]interface{}))
			}
			// Merge the nodes into the graph
			params := map[string]interface{}{
				"batch": userList,
			}
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, UserCreateQuery, params, neo4j.EagerResultTransformer,
				neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error merging user nodes: %v", err)
			}

			offset++
		} else {
			break
		}
	}

	log.Info("Finished processing users")
	return nil
}
