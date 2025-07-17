package core

import (
	"context"

	"github.com/Jeffail/gabs/v2"
	"github.com/neo4j/neo4j-go-driver/v5/neo4j"
	log "github.com/sirupsen/logrus"
)

func (i *CirroIngestor) ProcessKeyVaults() error {
	const resource_type = "microsoft.keyvault/vaults"

	var properties = []string{
		"id",
		"kind",
		"location",
		"name",
		"properties",
		"type",
	}

	const query = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:ArmResource {id: row.id})
		SET obj:KeyVault:ArmResource
		SET obj += {
			id: row.id,
			kind: row.kind,
			location: row.location,
			name: row.name,
			enableRbacAuthorization: row.properties.enableRbacAuthorization,
			enableSoftDelete: row.properties.enableSoftDelete,
			enabledForDeployment: row.properties.enabledForDeployment,
			publicNetworkAccess: row.properties.publicNetworkAccess,
			softDeleteRetentionInDays: row.properties.softDeleteRetentionInDays,
			vaultUri: row.properties.vaultUri,
			type: row.type
		}
	FOREACH  (accessPolicy in row.properties.accessPolicies |
		MERGE (p:GraphObject {id: accessPolicy.objectId})
		MERGE (p)-[r:HAS_POLICY]->(obj)
		SET r += {
			certificates: accessPolicy.permissions.certificates,
			keys: accessPolicy.permissions.keys,
			secrets: accessPolicy.permissions.secrets
		}
	)
	WITH obj, row
		MERGE (rg:ResourceGroup {id: row.rg_id})
		MERGE (rg)-[:CONTAINS]->(obj)
	`

	return i.ProcessSpecificArmResource(query, properties, resource_type)

}

func (i *CirroIngestor) ProcessVaultCertsTable() {

	const query = `/*cypher*/
	UNWIND $batch as row
	WITH row
		MERGE (obj:Certificate {thumbprint: row.thumbprint})
		SET obj += {
			thumbprint: row.thumbprint,
			cert: row.certdata.cer,
			created: apoc.date.format(row.attributes.created, 'ms'),
			expires: apoc.date.format(row.attributes.exp, 'ms'),
			notBefore: apoc.date.format(row.attributes.nbf, 'ms'),
			updated: apoc.date.format(row.attributes.updated, 'ms'),
			issuer: row.certdata.policy.issuer.name,
			exportable: row.certdata.policy.key_props.exportable,
			keyType: row.certdata.policy.key_props.kty,
			keyUsage: row.certdata.policy.x509_props.key_usage,
			subject: row.certdata.policy.x509_props.subject,
			keyContentType: row.keydata.contentType,
			key: row.keydata.value
		}
	WITH obj, row
		MERGE (vault:KeyVault {id: row.vault_id})
		MERGE (vault)-[:HAS_CERT]->(obj)
	`

	// If the vault_certs table doesn't exist or is empty, return
	countRow := i.DB.QueryRow(`SELECT COUNT(*) FROM vault_certs`)
	var count int
	err := countRow.Scan(&count)
	if err != nil {
		return
	}
	if count == 0 {
		return
	}
	log.Infof("Processing %d certificates from vault certs table", count)

	// Set limit and offset
	limit := 5000
	offset := 0

	for {
		var resourceList []map[string]interface{}

		if offset*limit <= count {
			rows, err := i.DB.Query(`SELECT thumbprint, vault_id, json(certdata), json(keydata) FROM vault_certs LIMIT ? OFFSET ?`, limit, offset*limit)
			if err != nil {
				log.Errorf("Error querying vault_certs: %s", err)
				return
			}
			defer rows.Close()

			for rows.Next() {
				var thumbprint, vault_id string
				var certdata, keydata []byte
				err := rows.Scan(&thumbprint, &vault_id, &certdata, &keydata)
				if err != nil {
					log.Errorf("Error scanning vault_certs: %s", err)
					continue
				}

				certJson, err := gabs.ParseJSON(certdata)
				if err != nil {
					log.Errorf("Error marshalling certdata: %s", err)
					continue
				}
				keyJson, err := gabs.ParseJSON(keydata)
				if err != nil {
					log.Errorf("Error marshalling keydata: %s", err)
					continue
				}

				cert := map[string]interface{}{
					"thumbprint": thumbprint,
					"vault_id":   vault_id,
					"certdata":   certJson.Data().(map[string]interface{}),
					"keydata":    keyJson.Data().(map[string]interface{}),
				}
				resourceList = append(resourceList, cert)
			}

			// Process the certificates
			log.Debugf("Processing %d certificates from offset %d", len(resourceList), offset)
			// log.Infof("resourceList: %v", resourceList)
			_, err = neo4j.ExecuteQuery(context.Background(), i.Driver, query, map[string]interface{}{"batch": resourceList}, neo4j.EagerResultTransformer, neo4j.ExecuteQueryWithDatabase("neo4j"))
			if err != nil {
				log.Errorf("Error processing certificates: %v", err)
			}
			offset++
		} else {
			break
		}
	}
	log.Info("Finished processing vault certs")
}
