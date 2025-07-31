use std::vec;

use crate::errors::CirroIngestError;
use crate::ingest::ingestor::CirroIngestor;
use log::{debug, error, info};
use neo4rs::{BoltType, query};
use serde_json::Value;
use uuid;

impl CirroIngestor {
    /// Processes graph objects from the database and inserts them into the graph
    async fn process_graph_objects(
        &self,
        table_name: &str,
        node_insert_query: &str,
        properties: Vec<&str>,
        post_queries: Option<Vec<&str>>,
    ) -> Result<(), CirroIngestError> {
        // Get the count of objects in the database
        let count = self
            .sql_conn
            .as_ref()
            .unwrap()
            .query_row(
                format!("SELECT COUNT(*) FROM {}", table_name).as_str(),
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0);

        if count == 0 {
            debug!("No {} found", table_name);
            return Ok(());
        }
        info!("Processing {:>5} {}", count, table_name);

        let limit = 5000;
        let mut offset = 0;

        loop {
            // If the offset exceeds the count, break the loop
            if offset * limit <= count {
                let select_query = format!(
                    "SELECT data FROM {} LIMIT {} OFFSET {}",
                    table_name,
                    limit,
                    offset * limit
                );
                debug!("Executing query: {}", select_query);

                let mut stmt = self
                    .sql_conn
                    .as_ref()
                    .unwrap()
                    .prepare(&select_query)
                    .unwrap();

                // Execute the query and process each row
                let rows = stmt.query_map([], |row| row.get(0))?;

                let processed_values: Vec<Value> = rows
                    .map(|result| {
                        result.map(|data: String| {
                            // Deserialize the JSON string into a Value
                            let value: Value = serde_json::from_str(&data)
                                .unwrap_or(Value::Object(serde_json::Map::new()));

                            // Create a new Value to hold the processed data
                            let mut new_value = Value::Object(serde_json::Map::new());

                            // For each property, set the value in the new Value
                            for property in &properties {
                                if let Some(val) = value.pointer(property) {
                                    new_value.as_object_mut().unwrap().insert(
                                        property.trim_start_matches('/').to_string(),
                                        val.clone(),
                                    );
                                }
                            }
                            debug!("Processed value: {:?}", new_value);
                            new_value
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Insert the processed values into the graph
                if !processed_values.is_empty() {
                    debug!(
                        "Inserting {} {} from offset {}",
                        processed_values.len(),
                        table_name,
                        offset
                    );
                    let _ = self
                        .graph
                        .run(query(node_insert_query).param(
                            "batch",
                            BoltType::try_from(serde_json::to_value(&processed_values)?)?,
                        ))
                        .await
                        .map_err(|e| CirroIngestError::DatabaseError(e.to_string()))?;

                    // if post_queries is Some
                    match &post_queries {
                        Some(queries) => {
                            debug!("Running post queries for {}", table_name);
                            for post_query in queries {
                                let _ =
                                    self.graph.run(query(post_query)).await.map_err(|e| {
                                        CirroIngestError::DatabaseError(e.to_string())
                                    })?;
                            }
                        }
                        None => debug!("No post queries to run for {}", table_name),
                    }
                } else {
                    error!(
                        "No values to insert for {} at offset {}",
                        table_name, offset
                    );
                }
                offset += 1;
                debug!("Processed {} {}s", &processed_values.len(), table_name);
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Processes graph applications
    pub async fn process_graph_applications(&self) -> Result<(), CirroIngestError> {
        // JSON Pointer style properties for the object
        let properties = vec![
            "/displayName",
            "/id",
            "/appId",
            "/passwordCredentials",
            "/publisherDomain",
            "/signInAudience",
            "/keyCredentials",
            "/owners",
            "/appRoles",
            "/appRoleAssignedTo",
        ];

        let node_insert_query = r#"
            UNWIND $batch as row
            WITH row
                MERGE (obj:GraphObject {id: row.id})
                SET obj:GraphApplication
                SET obj += {
                    displayName: row.displayName,
                    appId: row.appId,
                    publisherDomain: row.publisherDomain,
                    signInAudience: row.signInAudience
                }

            // App Roles
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.appRoles, []) AS appRole
                    MERGE (ar:GraphAppRole {id: appRole.id})
                    SET ar += {
                        displayName: appRole.displayName,
                        description: appRole.description,
                        allowedMemberTypes: appRole.allowedMemberTypes,
                        isEnabled: appRole.isEnabled,
                        isPreAuthorizationRequired: appRole.isPreAuthorizationRequired,
                        isPrivate: appRole.isPrivate,
                        origin: appRole.origin,
                        value: appRole.value
                    }
                    MERGE (obj)-[:HAS_APPROLE]->(ar)
                    RETURN count(*) AS _
                }
            
            // AppRoleAssignedTo
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.appRoleAssignedTo, []) AS assignedTo
                    MERGE (o:GraphObject {id: assignedTo.principalId})
                    MERGE (o)-[a:APPROLE {appRoleId: assignedTo.appRoleId}]->(obj)
                    RETURN count(*) AS _
                }

            // Unwind owners
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.owners, []) AS owner
                    MERGE (o:GraphObject {id: owner})
                    MERGE (o)-[:OWNS]->(obj)
                    RETURN count(*) AS _
                }

            // Unwind password credentials
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.passwordCredentials, []) AS passwordCredential
                    MERGE (p:ClientSecret {keyId: passwordCredential.keyId})
                    SET p += {
                        customKeyIdentifier: passwordCredential.customKeyIdentifier,
                        keyId: passwordCredential.keyId,
                        displayName: passwordCredential.displayName,
                        startDateTime: passwordCredential.startDateTime,
                        endDateTime: passwordCredential.endDateTime,
                        hint: passwordCredential.hint,
                        secretText: passwordCredential.secretText
                    }
                    MERGE (p)-[:AUTHENTICATES]->(obj)
                    RETURN count(*) AS _
                }
                
            // Unwind key credentials
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce([c IN coalesce(row.keyCredentials, []) WHERE c.customKeyIdentifier IS NOT NULL], []) AS keyCredential
                    MERGE (k:Certificate {thumbprint: keyCredential.customKeyIdentifier})
                    SET k += {
                        thumbprint: keyCredential.customKeyIdentifier,
                        displayName: keyCredential.displayName,
                        startDateTime: keyCredential.startDateTime,
                        endDateTime: keyCredential.endDateTime,
                        type: keyCredential.type,
                        usage: keyCredential.usage
                    }
                    MERGE (k)-[:AUTHENTICATES]->(obj)
                    RETURN count(*) AS _
                }
            "#;

        let app_role_post_query = r#"
            // Find all APPROLE relationships
            MATCH (:GraphObject)-[r:APPROLE]->(o:GraphServicePrincipal)-[:HAS_APPROLE]->(ar:GraphAppRole)
            WHERE r.appRoleId = ar.id
            SET r += {
                displayName: ar.displayName,
                description: ar.description,
                isEnabled: ar.isEnabled,
                isPreAuthorizationRequired: ar.isPreAuthorizationRequired,
                isPrivate: ar.isPrivate,
                origin: ar.origin,
                value: ar.value
            }
        "#;

        // Process the graph objects
        self.process_graph_objects(
            "applications",
            node_insert_query,
            properties,
            Some(vec![app_role_post_query]),
        )
        .await?;

        Ok(())
    }

    /// Processes administrative units
    pub async fn process_graph_administrative_units(&self) -> Result<(), CirroIngestError> {
        // JSON Pointer style properties for the object
        let properties = vec![
            "/id",
            "/description",
            "/displayName",
            "/isMemberManagementRestricted",
            "/members",
            "/membershipRule",
            "/membershipRuleProcessingState",
            "/membershipType",
            "/scopedRoleMembers",
            "/visibility",
        ];

        let node_insert_query = r#"
            UNWIND $batch as row
            WITH row
                MERGE (obj:GraphObject {id: row.id})
                SET obj:GraphAdministrativeUnit
                SET obj += {
                    description: row.description,
                    displayName: row.displayName,
                    isMemberManagementRestricted: row.isMemberManagementRestricted,
                    membershipRule: row.membershipRule,
                    membershipRuleProcessingState: row.membershipRuleProcessingState,
                    membershipType: row.membershipType,
                    visibility: row.visibility
                }

            // Unwind members
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.members, []) AS member
                    MERGE (m:GraphObject {id: member})
                    MERGE (m)-[:MEMBER_OF]->(obj)
                    RETURN count(*) AS _
                }

            // Unwind scoped role members
            WITH row, obj
                CALL {
                    WITH row, obj
                    WITH obj, coalesce(row.scopedRoleMembers, []) AS scopedRoleMembers
                        UNWIND scopedRoleMembers AS memberObj
                        MERGE (member:GraphObject {id: memberObj.roleMemberInfo_id})
                        MERGE (member)-[r:HAS_ROLE]->(obj)
                        SET r.roleId = memberObj.roleId

                        WITH obj, memberObj, member, r
                            OPTIONAL MATCH (role:GraphObject) WHERE role.id = memberObj.roleId
                            WITH obj, memberObj, member, r, role
                            WHERE role IS NOT NULL
                            SET r.roleName = role.displayName
                    RETURN count(*) AS _
                }
        "#;

        // Process the graph objects
        self.process_graph_objects("administrativeUnits", node_insert_query, properties, None)
            .await?;

        Ok(())
    }

    /// Processes graph devices
    pub async fn process_graph_devices(&self) -> Result<(), CirroIngestError> {
        // JSON Pointer style properties for the object
        let properties = vec![
            "/displayName",
            "/id",
            "/accountEnabled",
            "/deviceId",
            "/isCompliant",
            "/isManaged",
            "/manufacturer",
            "/memberOf",
            "/model",
            "/onPremisesLastSyncDateTime",
            "/onPremisesSyncEnabled",
            "/operatingSystem",
            "/operatingSystemVersion",
            "/profileType",
            "/trustType",
            "/registeredUsers",
            "/registeredOwners",
        ];

        let node_insert_query = r#"
            UNWIND $batch as row
            WITH row
                MERGE (obj:GraphObject {id: row.id})
                SET obj:GraphDevice
                SET obj += {
                    displayName: row.displayName,
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

            // Unwind memberOf
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.memberOf, []) AS groupId
                    MERGE (g:GraphObject {id: groupId})
                    MERGE (obj)-[:MEMBER_OF]->(g)
                    RETURN count(*) AS _
                }

            // Unwind registered users
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.registeredUsers, []) AS user
                    MERGE (u:GraphObject {id: user})
                    MERGE (u)-[:REGISTERED_USER]->(obj)
                    RETURN count(*) AS _
                }

            // Unwind registered owners
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.registeredOwners, []) AS owner
                    MERGE (o:GraphObject {id: owner})
                    MERGE (o)-[:REGISTERED_OWNER]->(obj)
                    RETURN count(*) AS _
                }

        "#;

        // Process the graph objects
        self.process_graph_objects("devices", node_insert_query, properties, None)
            .await?;
        Ok(())
    }

    /// Processes graph groups
    pub async fn process_graph_groups(&self) -> Result<(), CirroIngestError> {
        // JSON Pointer style properties for the object
        let properties = vec![
            "/displayName",
            "/id",
            "/groupTypes",
            "/owners",
            "/members",
            "/memberOf",
            "/membershipRule",
            "/membershipRuleProcessingState",
            "/onPremisesSecurityIdentifier",
            "/organizationId",
            "/securityEnabled",
            "/visibility",
        ];

        let node_insert_query = r#"
            UNWIND $batch as row
            WITH row
                MERGE (obj:GraphObject {id: row.id})
                SET obj:GraphGroup
                SET obj += {
                    displayName: row.displayName,
                    groupTypes: row.groupTypes,
                    membershipRule: row.membershipRule,
                    membershipRuleProcessingState: row.membershipRuleProcessingState,
                    onPremisesSecurityIdentifier: row.onPremisesSecurityIdentifier,
                    organizationId: row.organizationId,
                    securityEnabled: row.securityEnabled,
                    visibility: row.visibility
                }

            // Unwind memberOf
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.memberOf, []) AS groupId
                    MERGE (g:GraphObject {id: groupId})
                    MERGE (obj)-[:MEMBER_OF]->(g)
                    RETURN count(*) AS _
                }

            // Unwind owners
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.owners, []) AS owner
                    MERGE (o:GraphObject {id: owner})
                    MERGE (o)-[:OWNS]->(obj)
                    RETURN count(*) AS _
                }
            
            // Unwind members
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.members, []) AS member
                    MERGE (m:GraphObject {id: member})
                    MERGE (m)-[:MEMBER_OF]->(obj)
                    RETURN count(*) AS _
                }
        "#;

        // Process the graph objects
        self.process_graph_objects("groups", node_insert_query, properties, None)
            .await?;

        Ok(())
    }

    /// Processes graph roles
    pub async fn process_graph_roles(&self) -> Result<(), CirroIngestError> {
        let properties = vec![
            "/displayName",
            "/id",
            "/description",
            "/roleTemplateId",
            "/members",
        ];
        let node_insert_query = r#"
            UNWIND $batch as row
            WITH row
                MERGE (obj:GraphObject {id: row.id})
                SET obj:GraphRole
                SET obj += {
                    displayName: row.displayName,
                    description: row.description,
                    roleTemplateId: row.roleTemplateId
                }

            // Unwind members
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.members, []) AS member
                    MERGE (m:GraphObject {id: member})
                    MERGE (m)-[:HAS_ROLE]->(obj)
                    RETURN count(*) AS _
                }
        "#;

        // Process the graph objects
        self.process_graph_objects("directoryRoles", node_insert_query, properties, None)
            .await?;

        Ok(())
    }

    /// Processes graph service principals
    pub async fn process_graph_service_principals(&self) -> Result<(), CirroIngestError> {
        let properties = vec![
            "/accountEnabled",
            "/alternativeNames",
            "/appId",
            "/appOwnerOrganizationId",
            "/appRoleAssignedTo",
            "/appRoleAssignmentRequired",
            "/appRoles",
            "/displayName",
            "/id",
            "/keyCredentials",
            "/memberOf",
            "/owners",
            "/passwordCredentials",
            "/publisherName",
            "/replyUrls",
            "/servicePrincipalNames",
            "/servicePrincipalType",
        ];
        let node_insert_query = r#"
            UNWIND $batch as row

            MERGE (obj:GraphObject {id: row.id})
            SET obj:GraphServicePrincipal
            SET obj += {
                displayName: row.displayName,
                accountEnabled: row.accountEnabled,
                alternativeNames: row.alternativeNames,
                appId: row.appId,
                appOwnerOrganizationId: row.appOwnerOrganizationId,
                publisherName: row.publisherName,
                servicePrincipalType: row.servicePrincipalType,
                replyUrls: row.replyUrls,
                servicePrincipalNames: row.servicePrincipalNames
            }

            WITH row, obj 
                OPTIONAL MATCH (a:GraphApplication {appId: row.appId})
                CALL {
                    WITH obj, a
                    FOREACH (_ IN CASE WHEN a IS NOT NULL THEN [1] ELSE [] END |
                        MERGE (a)-[:REPRESENTED_BY]->(obj)
                    )
                    RETURN count(*) AS _
                }

            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.memberOf, []) AS groupId
                    MERGE (g:GraphObject {id: groupId})
                    MERGE (obj)-[:MEMBER_OF]->(g)
                    RETURN count(*) AS _
                }

            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.appRoles, []) AS appRole
                    MERGE (ar:GraphAppRole {id: appRole.id})
                    SET ar += {
                        displayName: appRole.displayName,
                        description: appRole.description,
                        allowedMemberTypes: appRole.allowedMemberTypes,
                        isEnabled: appRole.isEnabled,
                        isPreAuthorizationRequired: appRole.isPreAuthorizationRequired,
                        isPrivate: appRole.isPrivate,
                        origin: appRole.origin,
                        value: appRole.value
                    }
                    MERGE (obj)-[:HAS_APPROLE]->(ar)
                    RETURN count(*) AS _
                }

            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.appRoleAssignedTo, []) AS assignedTo
                    MERGE (o:GraphObject {id: assignedTo.principalId})
                    MERGE (o)-[a:APPROLE {appRoleId: assignedTo.appRoleId}]->(obj)
                    RETURN count(*) AS _
                }

            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.owners, []) AS ownerId
                    MERGE (o:GraphObject {id: ownerId})
                    MERGE (o)-[:OWNS]->(obj)
                    RETURN count(*) AS _
                }

            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.passwordCredentials, []) AS pwd
                    MERGE (p:ClientSecret {keyId: pwd.keyId})
                    SET p += {
                        customKeyIdentifier: pwd.customKeyIdentifier,
                        keyId: pwd.keyId,
                        displayName: pwd.displayName,
                        startDateTime: pwd.startDateTime,
                        endDateTime: pwd.endDateTime,
                        hint: pwd.hint,
                        secretText: pwd.secretText
                    }
                MERGE (p)-[:AUTHENTICATES]->(obj)
                RETURN count(*) AS _
                }

            WITH row, obj, [c IN row.keyCredentials WHERE c.customKeyIdentifier IS NOT NULL] AS keyCredentials
            CALL {
                WITH obj, keyCredentials
                UNWIND keyCredentials AS keyCredential
                MERGE (k:Certificate {thumbprint: keyCredential.customKeyIdentifier})
                SET k += {
                    thumbprint: keyCredential.customKeyIdentifier,
                    displayName: keyCredential.displayName,
                    startDateTime: keyCredential.startDateTime,
                    endDateTime: keyCredential.endDateTime,
                    type: keyCredential.type,
                    usage: keyCredential.usage
                }
                MERGE (k)-[:AUTHENTICATES]->(obj)
                RETURN count(*) AS _
            }
            RETURN count(obj) AS processedCount
        "#;

        let app_role_post_query = r#"
            // Find all APPROLE relationships
            MATCH (:GraphObject)-[r:APPROLE]->(o:GraphServicePrincipal)-[:HAS_APPROLE]->(ar:GraphAppRole)
            WHERE r.appRoleId = ar.id
            SET r += {
                displayName: ar.displayName,
                description: ar.description,
                isEnabled: ar.isEnabled,
                isPreAuthorizationRequired: ar.isPreAuthorizationRequired,
                isPrivate: ar.isPrivate,
                origin: ar.origin,
                value: ar.value
            }
        "#;

        // Process the graph objects
        self.process_graph_objects(
            "servicePrincipals",
            node_insert_query,
            properties,
            Some(vec![app_role_post_query]),
        )
        .await?;

        Ok(())
    }

    /// Processes graph users
    pub async fn process_graph_users(&self) -> Result<(), CirroIngestError> {
        let properties = vec![
            "/accountEnabled",
            "/city",
            "/companyName",
            "/country",
            "/creationType",
            "/department",
            "/displayName",
            "/givenName",
            "/id",
            "/jobTitle",
            "/mail",
            "/mailNickname",
            "/memberOf",
            "/mobilePhone",
            "/officeLocation",
            "/onPremisesDistinguishedName",
            "/onPremisesSamAccountName",
            "/onPremisesSecurityIdentifier",
            "/onPremisesUserPrincipalName",
            "/refreshTokensValidFromDateTime",
            "/securityIdentifier",
            "/state",
            "/surname",
            "/userPrincipalName",
            "/userType",
        ];

        let node_insert_query = r#"
            UNWIND $batch as row
            WITH row
                MERGE (obj:GraphObject {id: row.id})
                SET obj:GraphUser
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
            
            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.memberOf, []) AS groupId
                    MERGE (g:GraphObject {id: groupId})
                    MERGE (obj)-[:MEMBER_OF]->(g)
                    RETURN count(*) AS _
                }

        "#;
        self.process_graph_objects("users", node_insert_query, properties, None)
            .await?;

        Ok(())
    }

    /// Processes graph organizations
    pub async fn process_graph_organizations(&self) -> Result<(), CirroIngestError> {
        let properties = vec![
            "/id",
            "/businessPhones",
            "/createdDateTime",
            "/displayName",
            "/modifiedDateTime",
            "/onPremisesLastSyncDateTime",
            "/onPremisesSyncEnabled",
            "/tenantType",
            "/verifiedDomains",
        ];

        let node_insert_query = r#"
            UNWIND $batch as row
            WITH row
                MERGE (obj:GraphOrg {id: row.id})
                SET obj += {
                    businessPhones: row.businessPhones,
                    createdDateTime: row.createdDateTime,
                    displayName: row.displayName,
                    modifiedDateTime: row.modifiedDateTime,
                    onPremisesLastSyncDateTime: row.onPremisesLastSyncDateTime,
                    onPremisesSyncEnabled: row.onPremisesSyncEnabled,
                    tenantType: row.tenantType
                }
                MERGE (tenant:Tenant {id: '/tenants/' + row.id})
                MERGE (obj)-[:ASSOCIATED_WITH]->(tenant)
                MERGE (tenant)-[:ASSOCIATED_WITH]->(obj)

            WITH row, obj
                CALL {
                    WITH row, obj
                    UNWIND coalesce(row.verifiedDomains, []) as domain
                    MERGE (d:VerifiedDomain {name: domain.name})
                    SET d += {
                        isDefault: domain.isDefault,
                        isInitial: domain.isInitial,
                        type: domain.type
                    }
                    MERGE (obj)-[:VERIFIED_DOMAIN]->(d)
                    RETURN count(*) AS _
                }

        "#;
        self.process_graph_objects("organization", node_insert_query, properties, None)
            .await?;

        Ok(())
    }

    /// Process policies
    pub async fn process_graph_policies(&self) -> Result<(), CirroIngestError> {
        // For policies, we list all properties for all policy types
        // Not ideal, but the most straightforward approach given how we
        // need to handle different policy types and their specific properties.
        let properties = vec![
            "/id",
            "/displayName",
            "/description",
            // Authorization Policy
            "/allowEmailVerifiedUsersToJoinOrganization",
            "/allowInvitesFrom",
            "/allowUserConsentForRiskyApps",
            "/allowedToSignUpEmailBasedSubscriptions",
            "/allowedToUseSSPR",
            "/blockMsolPowerShell",
            "/defaultUserRolePermissions",
        ];

        let node_insert_query = r#"
            UNWIND $batch as row
            WITH row
                MERGE (obj:GraphPolicy {id: row._uniq_id})
                WITH obj, row WHERE row._policy_type = "authorizationpolicy"
                    SET obj += {
                        displayName: row.displayName,
                        allowEmailVerifiedUsersToJoinOrganization: row.allowEmailVerifiedUsersToJoinOrganization,
                        allowInvitesFrom: row.allowInvitesFrom,
                        allowUserConsentForRiskyApps: row.allowUserConsentForRiskyApps,
                        allowedToSignUpEmailBasedSubscriptions: row.allowedToSignUpEmailBasedSubscriptions,
                        allowedToUseSSPR: row.allowedToUseSSPR,
                        blockMsolPowerShell: row.blockMsolPowerShell,
                        allowedToCreateApps: row.defaultUserRolePermissions.allowedToCreateApps,
                        allowedToCreateSecurityGroups: row.defaultUserRolePermissions.allowedToCreateSecurityGroups,
                        allowedToCreateTenants: row.defaultUserRolePermissions.allowedToCreateTenants,
                        allowedToReadBitlockerKeysForOwnedDevice: row.defaultUserRolePermissions.allowedToReadBitlockerKeysForOwnedDevice,
                        allowedToReadOtherUsers: row.defaultUserRolePermissions.allowedToReadOtherUsers,
                        type: row._policy_type
                    }
                    MERGE (org:GraphOrg {id: row._tenant_id})
                    MERGE (org)-[:HAS_POLICY]->(obj)
        "#;
        // Get the count of objects in the database
        let count = self
            .sql_conn
            .as_ref()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM policies", [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap_or(0);

        if count == 0 {
            debug!("No policies found");
            return Ok(());
        }
        info!("Processing {:>5} policies", count);

        let limit = 5000;
        let mut offset = 0;

        loop {
            // If the offset exceeds the count, break the loop
            if offset * limit <= count {
                let select_query = format!(
                    "SELECT id, data FROM {} LIMIT {} OFFSET {}",
                    "policies",
                    limit,
                    offset * limit
                );
                debug!("Executing query: {}", select_query);

                let mut stmt = self
                    .sql_conn
                    .as_ref()
                    .unwrap()
                    .prepare(&select_query)
                    .unwrap();

                // Execute the query and process each row
                let rows = stmt.query_map([], |row| {
                    let id: String = row.get(0)?;
                    let data: String = row.get(1)?;
                    Ok((id, data))
                })?;

                let processed_values: Vec<Value> = rows
                    .filter_map(|result| {
                        match result {
                            Ok((id, data)) => {
                                // Deserialize the JSON string into a Value
                                let value: Value = serde_json::from_str(&data)
                                    .unwrap_or(Value::Object(serde_json::Map::new()));

                                // Create a new Value to hold the processed data
                                let mut new_value = Value::Object(serde_json::Map::new());

                                // For each property, set the value in the new Value
                                for property in &properties {
                                    if let Some(val) = value.pointer(property) {
                                        new_value.as_object_mut().unwrap().insert(
                                            property.trim_start_matches('/').to_string(),
                                            val.clone(),
                                        );
                                    }
                                }

                                // Extract tenant_id and policy_type from id
                                let mut parts = id.split('_');
                                let tenant_id = parts.next().unwrap_or("");
                                let policy_type = parts.next().unwrap_or("");

                                // Validate tenant_id is a GUID
                                if uuid::Uuid::parse_str(tenant_id).is_err() {
                                    error!("Invalid tenant_id: {}", tenant_id);
                                    return None;
                                }

                                // Add the tenant ID to the new value
                                new_value.as_object_mut().unwrap().insert(
                                    "_tenant_id".to_string(),
                                    Value::String(tenant_id.to_string()),
                                );

                                // Add the policy type to the new value
                                new_value.as_object_mut().unwrap().insert(
                                    "_policy_type".to_string(),
                                    Value::String(policy_type.to_string()),
                                );

                                // Replace the id in the JSON with the id
                                // There's no unique identifier for the original data, so we use the id we made
                                // Unique ID = tenantID_policyId
                                new_value.as_object_mut().unwrap().insert(
                                    "_uniq_id".to_string(),
                                    Value::String(id.to_lowercase()),
                                );
                                Some(new_value)
                            }
                            Err(e) => {
                                error!("Error processing row: {:?}", e);
                                None
                            }
                        }
                    })
                    .collect();

                // Insert the processed values into the graph
                if !processed_values.is_empty() {
                    debug!(
                        "Inserting {} policies from offset {}",
                        processed_values.len(),
                        offset
                    );
                    let _ = self
                        .graph
                        .run(query(node_insert_query).param(
                            "batch",
                            BoltType::try_from(serde_json::to_value(&processed_values)?)?,
                        ))
                        .await
                        .map_err(|e| CirroIngestError::DatabaseError(e.to_string()))?;
                } else {
                    error!("No values to insert for policies at offset {}", offset);
                }
                offset += 1;
                debug!("Processed {} policies", &processed_values.len());
            } else {
                break;
            }
        }
        Ok(())
    }
}
