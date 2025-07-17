package collectors

import (
	"fmt"

	"github.com/Jeffail/gabs/v2"
	"github.com/bishopfox/cirro/collector/credentials"
	log "github.com/sirupsen/logrus"
	_ "modernc.org/sqlite"
)

func EnrichConditionalAccessPolicies(ctx *CollectorContext) error {

	if err := ctx.OutputDB.Exec("CREATE TABLE IF NOT EXISTS caps (id TEXT PRIMARY KEY, data BLOB)"); err != nil {
		return err
	}

	var token *credentials.Token
	var response *gabs.Container
	var err error

	// Enrich the Conditional Access Policies
	if token, err = ctx.AadGraphCred.GetToken(); err != nil {
		log.Warn(err)
	}

	if token == nil {
		return fmt.Errorf("AAD Graph token is nil")
	}

	log.Info("Enriching Conditional Access Policies via AAD Graph")
	// Query the Conditional Access Policies via AAD Graph
	tenantId, err := token.GetClaim("tid")
	if err != nil {
		return err
	} else {

		uri := fmt.Sprintf("/%s/policies?api-version=1.61-internal", tenantId)
		response, _, err = pagedRequest(ctx.AadGraphCred, ctx.CloudEndpoints.AadGraphUrl, uri)
		if err != nil {
			return err
		}
	}

	if response != nil {
		values := response.Path("value").Children()
		for _, value := range values {
			// Insert the Conditional Access Policy into the database
			if err := ctx.OutputDB.Exec("REPLACE INTO caps (id, data) VALUES (?, jsonb(?));", value.Path("objectId").Data().(string), value.String()); err != nil {
				log.Error(err)
			}
		}
	}

	return nil
}
