use crate::errors::CirroIngestError;
use crate::ingest::ingestor::CirroIngestor;
use log::{debug, error, info};
use neo4rs::{BoltType, query};
use serde_json::Value;

impl CirroIngestor {
    /// Processes graph objects from the database and inserts them into the graph
    async fn process_graph_objects(
        &self,
        table_name: &str,
        node_insert_query: &str,
        properties: Vec<&str>,
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
                
            // Unwind owners
            WITH row, obj WHERE row.owners IS NOT NULL
                UNWIND row.owners AS owner
                MERGE (o:GraphObject {id: owner})
                MERGE (o)-[:OWNS]->(obj)

            // Unwind password credentials
            WITH row, obj WHERE row.passwordCredentials IS NOT NULL
                UNWIND row.passwordCredentials AS passwordCredential
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
                
            // Unwind key credentials
            WITH row, obj,  [c IN row.keyCredentials WHERE c.customKeyIdentifier IS NOT NULL] AS keyCredentials
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
        "#;

        // Process the graph objects
        self.process_graph_objects("applications", node_insert_query, properties)
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
            WITH row, obj WHERE row.members IS NOT NULL
                UNWIND row.members AS member
                MERGE (m:GraphObject {id: member})
                MERGE (m)-[:MEMBER_OF]->(obj)

            // Unwind scoped role members
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
        "#;

        // Process the graph objects
        self.process_graph_objects("administrativeUnits", node_insert_query, properties)
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
            // Unwind registered users
            WITH row, obj WHERE row.registeredUsers IS NOT NULL
                UNWIND row.registeredUsers AS user
                MERGE (u:GraphObject {id: user})
                MERGE (u)-[:REGISTERED_USER]->(obj)

            // Unwind registered owners
            WITH row, obj WHERE row.registeredOwners IS NOT NULL
                UNWIND row.registeredOwners AS owner
                MERGE (o:GraphObject {id: owner})
                MERGE (o)-[:REGISTERED_OWNER]->(obj)

        "#;

        // Process the graph objects
        self.process_graph_objects("devices", node_insert_query, properties)
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
            "/membershipRule",
            "/membershipRuleProcessingState",
            "/onPremisesSecurityIdentifier",
            "/organizationId",
            "/securityEnabled",
            "/visibility",
            "/owners",
            "/members",
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
            // Unwind owners
            WITH row, obj WHERE row.owners IS NOT NULL
                UNWIND row.owners AS owner
                MERGE (o:GraphObject {id: owner})
                MERGE (o)-[:OWNS]->(obj)
            
            // Unwind members
            WITH row, obj WHERE row.members IS NOT NULL
                UNWIND row.members AS member
                MERGE (m:GraphObject {id: member})
                MERGE (m)-[:MEMBER_OF]->(obj)
        "#;

        // Process the graph objects
        self.process_graph_objects("groups", node_insert_query, properties)
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
            WITH row, obj WHERE row.members IS NOT NULL
                UNWIND row.members AS member
                MERGE (m:GraphObject {id: member})
                MERGE (m)-[:HAS_ROLE]->(obj)
        "#;

        // Process the graph objects
        self.process_graph_objects("directoryRoles", node_insert_query, properties)
            .await?;

        Ok(())
    }

    /// Processes graph service principals
    pub async fn process_graph_service_principals(&self) -> Result<(), CirroIngestError> {
        let properties = vec![
            "/displayName",
            "/id",
            "/accountEnabled",
            "/alternativeNames",
            "/appId",
            "/appOwnerOrganizationId",
            "/publisherName",
            "/servicePrincipalType",
            "/replyUrls",
            "/servicePrincipalNames",
            "/keyCredentials",
            "/passwordCredentials",
            "/owners",
        ];
        let node_insert_query = r#"
            UNWIND $batch as row
            WITH row
                // If an application exists, link it to the service principal
                OPTIONAL MATCH (a:GraphApplication {appId: row.appId})
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

                WITH row, obj, a WHERE a IS NOT NULL
                    MERGE (a)-[:REPRESENTED_BY]->(obj)

            // Unwind owners
            WITH row, obj WHERE row.owners IS NOT NULL
                UNWIND row.owners AS owner
                MERGE (o:GraphObject {id: owner})
                MERGE (o)-[:OWNS]->(obj)

            // Unwind password credentials
            WITH row, obj WHERE row.passwordCredentials IS NOT NULL
                UNWIND row.passwordCredentials AS passwordCredential
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

            WITH row, obj, [c IN row.keyCredentials WHERE c.customKeyIdentifier IS NOT NULL] AS keyCredentials
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
        "#;

        // Process the graph objects
        self.process_graph_objects("servicePrincipals", node_insert_query, properties)
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
        "#;
        self.process_graph_objects("users", node_insert_query, properties)
            .await?;

        Ok(())
    }
}
