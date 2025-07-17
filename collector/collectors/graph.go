package collectors

import (
	"fmt"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/Jeffail/gabs/v2"
	log "github.com/sirupsen/logrus"
)

type GraphObjectType string

const (
	// Graph object types
	AdministrativeUnits GraphObjectType = "administrativeUnits"
	Applications        GraphObjectType = "applications"
	AppRoleAssignments  GraphObjectType = "appRoleAssignments"
	Devices             GraphObjectType = "devices"
	Groups              GraphObjectType = "groups"
	Roles               GraphObjectType = "directoryRoles"
	ServicePrincipals   GraphObjectType = "servicePrincipals"
	Users               GraphObjectType = "users"
)

type GraphObject struct {
	resource    GraphObjectType
	queryParams string
	expandProps map[string][]string // e.g.,  {"members": "id"} or {"scopedRoleMembers": ["roleId", "roleMemberInfo.id"]}
}

// expandObject expands the specified object type and appends values to the object
func expandObject(ctx *CollectorContext, resource GraphObjectType, object *gabs.Container, property string, json_xpath []string) {

	// Get the object ID
	objectID := object.Path("id").Data().(string)
	url := fmt.Sprintf("%s/beta/%s/%s/%s", ctx.CloudEndpoints.MsGraphUrl, resource, objectID, property)
ExpandLoop:
	for {
		// log.Info("Expanding " + url)
		response, err := pagedGraphRequest(ctx, url)
		if err != nil {
			log.Error(err)
			break
		}

		values := response.Path("value").Children()
		// Add the values to the object with the property name
		for _, value := range values {

			// Store the values in a map
			xpath_values := make(map[string]string)

			for _, xpath := range json_xpath {
				if value.ExistsP(xpath) {
					// Make a copy of xpath replacing . with _
					json_key := strings.Replace(xpath, ".", "_", -1)
					xpath_values[json_key] = value.Path(xpath).Data().(string)
				} else {
					log.Errorf("No %s value for property %s for %s - %s", json_xpath, property, resource, objectID)
					continue
				}
				object.ArrayAppend(xpath_values, property)
			}
		}

		// Gabs has issues with the @ in @odata namespace so do this instead
		responseMap := response.ChildrenMap()
		for key, value := range responseMap {
			if key == "@odata.nextLink" {
				url = value.Data().(string)
				log.Debug("Next page: " + url)
				continue ExpandLoop
			}
		}
		// No next page
		break
	}
}

// queryObjects queries the Graph API for the specified object type
func (gObject *GraphObject) queryObjects(ctx *CollectorContext) {

	// Create the table in the database
	// Allow these strings to be formatted because it's derived from the GraphObjectType
	tableCreateStatement := fmt.Sprintf(
		`CREATE TABLE IF NOT EXISTS %s (
		id TEXT PRIMARY KEY,
		data BLOB
		);`, gObject.resource,
	)
	tableInsertStatement := fmt.Sprintf(
		`REPLACE INTO %s (id, data) VALUES (?, jsonb(?));`, gObject.resource,
	)

	tableInsertPrepared, err := ctx.OutputDB.PrepareTx(tableInsertStatement)
	if err != nil {
		log.Error(err)
		return
	}

	if err := ctx.OutputDB.Exec(tableCreateStatement); err != nil {
		log.Error(err)
		return
	}

	log.Info("Querying Graph objects for " + gObject.resource)
	start := time.Now()
	nextUrl := fmt.Sprintf("/beta/%s?%s", gObject.resource, gObject.queryParams)

PageLoop:
	for {
		log.Debug("Querying " + nextUrl)
		response, _, err := pagedRequest(ctx.MSGraphCred, ctx.CloudEndpoints.MsGraphUrl, nextUrl)
		if err != nil {
			log.Error(err)
			break
		}

		if response.Exists("@odata.error") {
			log.Error(response.Path("@odata.error.message").Data().(string))
			break
		}

		values := response.Path("value").Children()
		resultWg := new(sync.WaitGroup)
		for _, value := range values {
			resultWg.Add(1)

			// Create a map interface to hold results

			go func(value *gabs.Container) {
				defer resultWg.Done()

				expandWg := new(sync.WaitGroup)
				// Expand properties
				for property, json_xpaths := range gObject.expandProps {
					ctx.GraphSemaphore <- true
					expandWg.Add(1)

					go func(property string) {
						defer expandWg.Done()
						expandObject(ctx, gObject.resource, value, property, json_xpaths)

						// Release the semaphore for this goroutine
						<-ctx.GraphSemaphore
					}(property)
				}
				expandWg.Wait()

				// Write the value to the database
				value_id := value.Path("id").Data().(string)
				data := value.String()
				if _, err := tableInsertPrepared.Exec(value_id, data); err != nil {
					log.Error(err)
				}
			}(value)

			// Add a count for this object type
			ctx.GraphCounterMap[gObject.resource]++
		}
		resultWg.Wait()
		// Gabs has issues with the @ in @odata namespace so do this instead
		responseMap := response.ChildrenMap()
		for key, value := range responseMap {
			if key == "@odata.nextLink" {
				nextUrl = value.Data().(string)
				log.Debug("Next page: " + nextUrl)
				continue PageLoop
			}
		}

		// No next page
		break
	}
	elapsed := time.Since(start)
	log.Infof("Found %d %s (%s)", ctx.GraphCounterMap[gObject.resource], gObject.resource, elapsed)
}

// pagedGraphRequest makes a request to the Graph API and returns the response
func pagedGraphRequest(ctx *CollectorContext, uri string) (*gabs.Container, error) {
	var requestUrl string

	graphUrl := ctx.CloudEndpoints.MsGraphUrl

	if strings.HasPrefix(uri, graphUrl) {
		requestUrl = uri
	} else {
		requestUrl = fmt.Sprintf("%s/beta/%s", graphUrl, uri)
	}
	log.Debug("Paged request to " + requestUrl)

	token, err := ctx.MSGraphCred.GetToken()
	if err != nil {
		return nil, err
	}

	request, err := http.NewRequest(http.MethodGet, requestUrl, nil)
	if err != nil {
		return nil, err
	}
	request.Header.Set("Authorization", "Bearer "+token.AccessToken)
	resp, err := http.DefaultClient.Do(request)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	parsed, err := gabs.ParseJSONBuffer(resp.Body)
	if err != nil {
		return nil, err
	}

	if parsed.Exists("@odata.error") {
		return nil, fmt.Errorf("%s", parsed.Path("@odata.error.message").Data().(string))
	}

	return parsed, nil
}

// enumerateGraph enumerates objects in Microsoft Graph
func enumerateGraph(ctx *CollectorContext) {
	log.Info("Enumerating Microsoft Graph")

	wg := new(sync.WaitGroup)

	userEnumerator := GraphObject{
		resource:    Users,
		queryParams: "$top=999",
		expandProps: map[string][]string{},
	}
	groupEnumerator := GraphObject{
		resource:    Groups,
		queryParams: "$top=999",
		expandProps: map[string][]string{},
	}
	applicationEnumerator := GraphObject{
		resource:    Applications,
		queryParams: "$top=999",
		expandProps: map[string][]string{"owners": {"id"}},
	}
	servicePrincipalEnumerator := GraphObject{
		resource:    ServicePrincipals,
		queryParams: "$top=999",
		expandProps: map[string][]string{"owners": {"id"}},
	}
	deviceEnumerator := GraphObject{
		resource:    Devices,
		queryParams: "$top=999",
		expandProps: map[string][]string{"registeredUsers": {"id"}},
	}
	roleEnumerator := GraphObject{
		resource:    Roles,
		queryParams: "",
		expandProps: map[string][]string{"members": {"id"}},
	}
	administrativeUnitEnumerator := GraphObject{
		resource:    AdministrativeUnits,
		queryParams: "",
		expandProps: map[string][]string{"members": {"id"}, "scopedRoleMembers": {"roleId", "roleMemberInfo.id"}},
	}

	for _, enumerator := range []GraphObject{
		userEnumerator,
		groupEnumerator,
		applicationEnumerator,
		servicePrincipalEnumerator,
		deviceEnumerator,
		roleEnumerator,
		administrativeUnitEnumerator,
	} {
		wg.Add(1)
		go func(e GraphObject) {
			defer wg.Done()
			e.queryObjects(ctx)
		}(enumerator)
	}
	wg.Wait()

	log.Info("Finished enumerating Microsoft Graph")
}
