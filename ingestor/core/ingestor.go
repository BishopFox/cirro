package core

import (
	"context"
	"database/sql"
	"fmt"

	_ "modernc.org/sqlite"

	"github.com/bishopfox/cirro/ingestor/schema/models"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j"
	neo4jlog "github.com/neo4j/neo4j-go-driver/v5/neo4j/log"
	log "github.com/sirupsen/logrus"
)

const (
	// Merge nodes and set labels
	MergeNodeQueryTemplate = `/*cypher*/
	UNWIND $batch as row
	CALL apoc.merge.node($labels, {id: row.id}, row, row) YIELD node as obj
	CALL apoc.create.setLabels(obj, $labels) YIELD node
	RETURN node
	`

	// Merge relationships passing in a batch of maps
	MergeRelationQueryTemplate = `
	UNWIND $batch AS row
		MATCH (from:$sourceLabel {id: row.sourceId})
    WITH row, from
		WHERE from IS NOT NULL
		MERGE (to:$targetLabel {name: row.targetId})
		MERGE (from)-[rel:$relationshipType]->(to)
		RETURN rel
`

	// Using sprintf to insert the label name since the driver doesn't support parameters for labels here
	// %[1]s is a nice way to say "insert the first parameter here"
	CreateConstraintQueryTemplate = "CREATE CONSTRAINT IF NOT EXISTS FOR (n: %s) REQUIRE n.id IS UNIQUE"
	CreateIndexQueryTemplate      = "CREATE INDEX %[1]s_id IF NOT EXISTS FOR (n: %[1]s) ON (n.id)"

	// Merge nodes with the same ID
	PostProcessMergeQuery = `MATCH (n)
	WITH lower(n.id) AS id, COLLECT(n) AS nodesToMerge
	WHERE id IS NOT NULL AND size(nodesToMerge) > 1
	CALL apoc.refactor.mergeNodes(nodesToMerge, {mergeRels:true})
	YIELD node
	RETURN count(*);`
)

type Neo4jConfig struct {
	Uri      string
	Username string
	Password string
}

type CirroIngestor struct {
	Neo4jConfig
	InputFile string
	Driver    neo4j.DriverWithContext
	Logger    neo4jlog.BoltLogger
	DB        *sql.DB
}

func NewCirroIngestor(inputPath string, username string, password string, server string) (*CirroIngestor, error) {
	config := Neo4jConfig{
		Uri:      server,
		Username: username,
		Password: password,
	}
	driver, err := neo4j.NewDriverWithContext(config.Uri, neo4j.BasicAuth(config.Username, config.Password, ""))
	if err != nil {
		return nil, err
	}
	return &CirroIngestor{
		Neo4jConfig: config,
		InputFile:   inputPath,
		Driver:      driver,
		Logger:      neo4jlog.BoltToConsole(),
	}, nil
}

func (ingestor *CirroIngestor) PostProcess() error {
	log.Info("Running post processing merge query")
	goCtx := context.Background()
	_, err := neo4j.ExecuteQuery(goCtx, ingestor.Driver, PostProcessMergeQuery, nil, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
	if err != nil {
		log.Error(err)
		return err
	}
	return nil
}

func (ingestor *CirroIngestor) StartIngestion() error {
	log.Infof("Processing database file: %s", ingestor.InputFile)

	// Open the database file
	db, err := sql.Open("sqlite", ingestor.InputFile)
	if err != nil {
		return err
	}
	ingestor.DB = db

	// Run processing functions
	processors := []func() error{
		ingestor.ProcessGraphUsers,
		ingestor.ProcessGraphApplications,
		ingestor.ProcessGraphDevices,
		ingestor.ProcessGraphServicePrincipals,
		ingestor.ProcessGraphGroups,
		ingestor.ProcessGraphRoles,
		ingestor.ProcessGraphAdministrativeUnits,
		ingestor.ProcessTenants,
		ingestor.ProcessSubscriptions,
		ingestor.ProcessResourceGroups,
		ingestor.ProcessArmResources,
		ingestor.ProcessAzureRbac,
		ingestor.ProcessAutomationAccounts,
		ingestor.ProcessAvailabilitySets,
		ingestor.ProcessAzureArcSqlServers,
		ingestor.ProcessClassicStorageAccounts,
		ingestor.ProcessCognitiveServicesAccount,
		ingestor.ProcessCommunicationServices,
		ingestor.ProcessContainerRegistries,
		ingestor.ProcessDnsZones,
		ingestor.ProcessDisks,
		ingestor.ProcessHybridMachines,
		ingestor.ProcessKeyVaults,
		ingestor.ProcessNetworkInterfaces,
		ingestor.ProcessNetworkSecurityGroups,
		ingestor.ProcessPublicIpAddresses,
		ingestor.ProcessRestorePointCollections,
		ingestor.ProcessSnapshots,
		ingestor.ProcessSSHPublicKeys,
		ingestor.ProcessStorageAccounts,
		ingestor.ProcessUserAssignedIdentities,
		ingestor.ProcessVirtualMachines,
		ingestor.ProcessVirtualNetworks,
	}

	enrichmentProcessors := []func(){
		ingestor.ProcessVaultCertsTable,
	}

	for _, processor := range processors {
		if err := processor(); err != nil {
			log.Error(err)
		}
		ingestor.PostProcess()
	}

	for _, processor := range enrichmentProcessors {
		processor()
	}
	return nil
}

func (ingestor *CirroIngestor) Run() error {
	goCtx := context.Background()
	log.Infof("Verifying connectivity to Neo4J at %s", ingestor.Uri)
	if err := ingestor.Driver.VerifyConnectivity(goCtx); err != nil {
		return err
	}
	defer ingestor.Driver.Close(goCtx)

	// Create constraints and indexes
	// These have to be created separately for each label so do it in a loop
	log.Info("Creating constraints and indexes for labels")
	for label := range models.NodeLabelToNodeMap {
		for _, query := range []string{CreateConstraintQueryTemplate, CreateIndexQueryTemplate} {
			_, err := neo4j.ExecuteQuery(goCtx, ingestor.Driver, fmt.Sprintf(query, label), nil, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Error(err)
				continue
			}
		}
	}

	// Process the results file
	err := ingestor.StartIngestion()
	if err != nil {
		log.Error(err)
	}

	// Run the post processing merge query
	log.Info("Running final post processing merge query")
	err = ingestor.PostProcess()
	if err != nil {
		log.Error(err)
		return err
	}
	log.Info("Ingestion complete")
	return nil
}
