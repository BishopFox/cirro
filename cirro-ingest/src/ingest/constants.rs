use strum_macros::EnumIter;

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum GraphType {
    Neo4j,
    Memgraph,
}

#[derive(Debug, Clone)]
pub struct CypherConstants {
    pub create_constraint_query: String,
    pub create_index_query: String,
    pub post_process_merge_query: String,
}

impl CypherConstants {
    pub fn new(graph_type: GraphType) -> Self {
        match graph_type {
            // Neo4j style constants for creating constraints and indexes
            // Neo4j creates indexes automatically when creating constraints
            // so the index creation queries are left empty.
            GraphType::Neo4j => Self {
                create_constraint_query:
                    "CREATE CONSTRAINT IF NOT EXISTS FOR (n: {}) REQUIRE n.id IS UNIQUE".to_string(),
                create_index_query: "CREATE INDEX {}_id IF NOT EXISTS FOR (n: {}) ON (n.id)"
                    .to_string(), // Neo4j creates indexes automatically with constraints
                post_process_merge_query: r#"MATCH (n)
	WITH n, lower(n.id) AS id, COLLECT(n) AS nodesToMerge
	WHERE id IS NOT NULL AND size(nodesToMerge) > 1
	CALL apoc.refactor.mergeNodes(nodesToMerge, {mergeRels:true})
	YIELD node
	RETURN count(*);"#
                    .to_string(),
            },

            // Memgraph style constants for creating constraints and indexes
            // Memgraph requires explicit index creation, so the index creation queries are provided.
            GraphType::Memgraph => Self {
                create_constraint_query: "CREATE CONSTRAINT ON (n: {}) ASSERT n.id IS UNIQUE"
                    .to_string(),
                create_index_query: "CREATE INDEX ON :{}(id);".to_string(),
                post_process_merge_query: r#"MATCH (n:GraphObject)
WITH toLower(n.id) AS id, collect(n) AS nodesToMerge
WHERE id IS NOT NULL AND size(nodesToMerge) > 1
CALL refactor.merge_nodes(nodesToMerge, {mergeRels:true})
YIELD node
RETURN count(*);"#
                    .to_string(),
            },
        }
    }
}

#[derive(strum_macros::Display, EnumIter, Debug, PartialEq)]
pub enum NodeType {
    GraphObject,
    GraphApplication,
    GraphDevice,
    GraphGroup,
    GraphRole,
    GraphServicePrincipal,
    GraphUser,
    ArmResource,
    ArcSqlServer,
    AutomationAccount,
    AvailabilitySet,
    AzureRbac,
    ClassicStorageAccount,
    ContainerRegistry,
    CommunicationServiceAccount,
    Disk,
    DnsZone,
    HybridMachine,
    IpConfiguration,
    KeyVault,
    ManagedIdentity,
    NetworkInterface,
    NetworkSecurityGroup,
    PublicIPAddress,
    ResourceGroup,
    RestorePointCollection,
    Runbook,
    Snapshot,
    SSHPublicKey,
    StorageAccount,
    Subnet,
    Subscription,
    Tenant,
    UserAssignedIdentity,
    VirtualNetwork,
    VirtualMachine,
}
