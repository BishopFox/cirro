package collectors

import (
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/Jeffail/gabs/v2"
	"github.com/bishopfox/cirro/collector/credentials"
	log "github.com/sirupsen/logrus"
)

var enumeratedRoleDefinitions sync.Map

type RequestThrottleEvent struct {
	mu     sync.Mutex
	cond   *sync.Cond
	active atomic.Bool // true = stop goroutines
}

func NewEvent() *RequestThrottleEvent {
	e := &RequestThrottleEvent{}
	e.mu = sync.Mutex{}
	e.cond = sync.NewCond(&e.mu)
	return e
}

func (e *RequestThrottleEvent) WaitIfSet() {
	e.mu.Lock()
	for e.active.Load() {
		e.cond.Wait()
	}
	e.mu.Unlock()
}

func (e *RequestThrottleEvent) Set() {
	e.active.Store(true)
}

func (e *RequestThrottleEvent) Clear() {
	e.mu.Lock()
	e.active.Store(false)
	e.cond.Broadcast() // wake up all waiting goroutines
	e.mu.Unlock()
}

func pagedARMRequest(cred credentials.AuthCredential, resourceUrl string, uri string, throttleEvent *RequestThrottleEvent) (*gabs.Container, error) {
	if throttleEvent != nil {
		throttleEvent.WaitIfSet()
	}
	response, headers, err := pagedRequest(cred, resourceUrl, uri)
	remaining_subscription_reads := headers.Get("x-ms-ratelimit-remaining-subscription-reads")
	remaining_subscription_global_reads := headers.Get("x-ms-ratelimit-remaining-subscription-global-reads")

	already_throttled := false

	if remaining_subscription_reads != "" {
		sub_reads_int, err := strconv.Atoi(remaining_subscription_reads)
		if err != nil {
			return nil, err
		}

		// Small buffer to prevent throttle issues
		if sub_reads_int <= 25 {
			// Check to see if the event has already been triggered.
			// This is to prevent multiple goroutines from triggering a sleep
			if !throttleEvent.active.Load() {
				throttleEvent.Set()
				log.Info("Throttling limit hit on subscription reads. Waiting 20 seconds...")
				time.Sleep(20 * time.Second)
				throttleEvent.Clear()

				already_throttled = true
			}

		}
	}

	// Check the global reads. If already throttled, don't check again
	if remaining_subscription_global_reads != "" && already_throttled {
		sub_global_reads_int, err := strconv.Atoi(remaining_subscription_global_reads)
		if err != nil {
			return nil, err
		}

		if sub_global_reads_int <= 20 {
			// Check to see if the event has already been triggered.
			// This is to prevent multiple goroutines from triggering a sleep
			if !throttleEvent.active.Load() {
				throttleEvent.Set()
				log.Info("Throttling limit hit on subscription global reads. Waiting 20 seconds...")
				time.Sleep(20 * time.Second)
				throttleEvent.Clear()
			}
		}
	}

	if err != nil {
		return nil, err
	}

	return response, err

}

func QueryResources(credential credentials.AuthCredential, baseUrl string, uri string, throttleEvent *RequestThrottleEvent) ([]map[string]interface{}, error) {
	var resources []map[string]interface{}

	nextUrl := fmt.Sprintf("%s/%s", baseUrl, uri)
PageLoop:
	for {
		log.Debug("Querying " + nextUrl)
		response, err := pagedARMRequest(credential, baseUrl, nextUrl, throttleEvent)
		if err != nil {
			return resources, err
		}

		if response.Exists("error") {
			log.Error(response.Path("error.message").Data().(string))
			return resources, fmt.Errorf("error querying resources: %s", response.Path("error.message").Data().(string))
		}

		if response.Exists("value") {
			values := response.Path("value").Children()

			if len(values) == 0 {
				// Either a single object or no objects
				// If it's a single object, append it to the resources
				// If it's no objects, make resources empty and break the loop
				if response.Exists("id") {
					resources = append(resources, response.Data().(map[string]interface{}))
				} else {
					resources = make([]map[string]interface{}, 0)
					break
				}
			} else {
				for _, value := range values {
					resources = append(resources, value.Data().(map[string]interface{}))
				}
			}
		} else {
			// This is a single object
			resources = append(resources, response.Data().(map[string]interface{}))
		}

		// Gabs has issues with the @ in @odata namespace so do this instead
		responseMap := response.ChildrenMap()
		for key, value := range responseMap {
			if key == "nextLink" {
				if value.Data() != nil {
					nextUrl = value.Data().(string)
					log.Debug("Next page: " + nextUrl)
					continue PageLoop
				}
			}
		}
		// No next page
		break
	}
	return resources, nil
}

func enumerateRoleAssignments(ctx *CollectorContext, scope string, throttleEvent *RequestThrottleEvent) []map[string]interface{} {
	var roles []map[string]interface{}

	uri := fmt.Sprintf("%s/providers/Microsoft.Authorization/roleAssignments?api-version=2022-04-01", scope)
	assignments, err := QueryResources(ctx.ARMCred, ctx.CloudEndpoints.ArmUrl, uri, throttleEvent)
	if err != nil && assignments == nil || len(assignments) == 0 {
		log.Warnf("No role assignments found for %s", scope)
		return roles
	}

	for _, assignment := range assignments {

		// Get the role definition id to get the definition
		roleDefinitionId := assignment["properties"].(map[string]interface{})["roleDefinitionId"].(string)[1:]
		uri := fmt.Sprintf("%s?disambiguation_dummy&api-version=2022-04-01", roleDefinitionId)

		// Check if we have already enumerated this role definition
		var response *gabs.Container
		var err error
		if roledef, ok := enumeratedRoleDefinitions.Load(roleDefinitionId); ok {
			// Already enumerated
			response = roledef.(*gabs.Container)
		} else {
			response, err = pagedARMRequest(ctx.ARMCred, ctx.CloudEndpoints.ArmUrl, uri, throttleEvent)
			if err != nil {
				log.Error(err)
				continue
			}
			if response.Exists("error") {
				log.Error(response.Path("error.message").Data().(string))
				continue
			}
			enumeratedRoleDefinitions.Store(roleDefinitionId, response)
		}

		// Expand the assignment with new fields
		assignment["permissions"] = response.Path("properties.permissions").Data().([]interface{})
		assignment["roleName"] = strings.ReplaceAll(response.Path("properties.roleName").Data().(string), " ", "")
		assignment["roleType"] = response.Path("properties.type").Data().(string)

		// Most role definitions have a description field, but maybe not custom ones.
		if response.Path("properties.description").Data() != nil {
			assignment["description"] = response.Path("properties.description").Data().(string)
		} else {
			assignment["description"] = ""
		}
		roles = append(roles, assignment)
	}
	return roles

}

func enumerateSubscription(ctx *CollectorContext, subscription map[string]interface{}, throttleEvent *RequestThrottleEvent) {

	subscriptionId := subscription["subscriptionId"].(string)
	subscriptionName := subscription["displayName"].(string)
	log.Infof("Enumerating subscription: %s - %s", subscriptionName, subscriptionId)

	// Enumerate role assignments
	// https://github.com/Azure/azure-rest-api-specs/tree/main/specification/authorization/resource-manager/Microsoft.Authorization/stable
	roleAssignments := enumerateRoleAssignments(ctx, subscription["id"].(string), throttleEvent)
	roleInsertStatement, err := ctx.OutputDB.PrepareTx("INSERT INTO roleAssignments (id, data) VALUES (?, jsonb(?))")
	if err != nil {
		log.Error(err)
	}

	for _, roleAssignment := range roleAssignments {
		roleAssignmentData, err := json.Marshal(roleAssignment)
		if err != nil {
			log.Error(err)
			continue
		}

		_, err = roleInsertStatement.Exec(roleAssignment["id"].(string), string(roleAssignmentData))
		if err != nil {
			log.Error(err)
		}

	}

	// Next make a call to get all the resource providers
	// https://docs.microsoft.com/en-us/rest/api/resources/providers/list
	// This is so we can enumerate all the resource types with the latest api version
	providers := make(map[string]string)
	uri := fmt.Sprintf("subscriptions/%s/providers?api-version=2022-12-01", subscriptionId)
	response, err := QueryResources(ctx.ARMCred, ctx.CloudEndpoints.ArmUrl, uri, throttleEvent)
	if err != nil && response == nil || len(response) == 0 {
		log.Warnf("Could not get providers for subscription: %s", subscriptionId)
		return
	}
	for _, provider := range response {

		namespace := provider["namespace"].(string)
		resourceTypes := provider["resourceTypes"].([]interface{})
		for _, resourceType := range resourceTypes {
			namespaceAndType := strings.ToLower(namespace + "/" + resourceType.(map[string]interface{})["resourceType"].(string))
			apiVersions := resourceType.(map[string]interface{})["apiVersions"].([]interface{})

			// Get the latest api version
			if len(apiVersions) > 0 {
				providers[namespaceAndType] = apiVersions[0].(string)
			}
		}
	}

	// Create the table for the subscription
	subscriptionInsertPrepared, err := ctx.OutputDB.PrepareTx("REPLACE INTO resources (id, sub_id, rg_id, resource_type, data) VALUES (?, ?, ?, ?, jsonb(?))")
	if err != nil {
		log.Error(err)
	}

	// Enumerate the resource groups
	// https://docs.microsoft.com/en-us/rest/api/resources/resourcegroups/list
	uri = fmt.Sprintf("subscriptions/%s/resourcegroups?api-version=2022-12-01", subscriptionId)
	resourceGroups, err := QueryResources(ctx.ARMCred, ctx.CloudEndpoints.ArmUrl, uri, throttleEvent)
	if err != nil && resourceGroups == nil || len(resourceGroups) == 0 {
		log.Warnf("Could not get resource groups for subscription: %s", subscriptionId)
		return
	}
	resourceGroupWg := new(sync.WaitGroup)
	for _, resourceGroup := range resourceGroups {
		rgData, err := json.Marshal(resourceGroup)
		if err != nil {
			log.Error(err)
			continue
		}

		// Insert the resource group into the subscription table
		_, err = subscriptionInsertPrepared.Exec(resourceGroup["id"].(string), subscriptionId, resourceGroup["id"].(string), resourceGroup["type"].(string), string(rgData))
		if err != nil {
			log.Error(err)
		}

		// Enumerate the resource group
		resourceGroupWg.Add(1)
		go func(resourceGroup map[string]interface{}) {
			defer resourceGroupWg.Done()
			enumerateResourceGroup(subscriptionName, resourceGroup, ctx, subscriptionId, providers, throttleEvent)
		}(resourceGroup)
	}
	resourceGroupWg.Wait()
	log.Infof("Finished enumerating subscription: %s - %s", subscriptionName, subscriptionId)
}

// Refactored to goroutine with VS Code. Lol.
func enumerateResourceGroup(subscriptionName string, resourceGroup map[string]interface{}, ctx *CollectorContext, subscriptionId string, providers map[string]string, throttleEvent *RequestThrottleEvent) {
	log.Infof("Enumerating %s : %s", subscriptionName, resourceGroup["name"].(string))

	roleAssignments := enumerateRoleAssignments(ctx, resourceGroup["id"].(string), throttleEvent)
	roleAssignmentsInsertPrepared, err := ctx.OutputDB.PrepareTx("REPLACE INTO roleAssignments (name, id, data) VALUES (?, ?, jsonb(?))")
	if err != nil {
		log.Error(err)
	}

	for _, roleAssignment := range roleAssignments {
		roleAssignmentData, err := json.Marshal(roleAssignment)
		if err != nil {
			log.Error(err)
			continue
		}
		_, err = roleAssignmentsInsertPrepared.Exec(roleAssignment["name"].(string), roleAssignment["id"].(string), string(roleAssignmentData))
		if err != nil {
			log.Error(err)
		}

	}

	// Enumerate the resources in the resource group
	resourceInsertPrepared, err := ctx.OutputDB.PrepareTx("REPLACE INTO resources (id, sub_id, rg_id, resource_type, data) VALUES (?, ?, ?, ?, jsonb(?))")
	if err != nil {
		log.Error(err)
	}

	uri := fmt.Sprintf("subscriptions/%s/resourcegroups/%s/resources?api-version=2023-07-01", subscriptionId, resourceGroup["name"].(string))
	resources, err := QueryResources(ctx.ARMCred, ctx.CloudEndpoints.ArmUrl, uri, throttleEvent)
	if err != nil {
		log.Warnf("Could not get resources for resource group %s: %v", resourceGroup["name"].(string), err)
		return
	}

	// If there are no resources, return
	if len(resources) == 0 {
		return
	}

	for _, listedResource := range resources {

		apiVersion := providers[strings.ToLower(listedResource["type"].(string))]

		resourceId := listedResource["id"].(string)
		uri := fmt.Sprintf("%s?api-version=%s", resourceId, apiVersion)
		response, err := pagedARMRequest(ctx.ARMCred, ctx.CloudEndpoints.ArmUrl, uri, throttleEvent)
		if err != nil {
			log.Error(err)
			continue
		}
		if response.Exists("error") {
			log.Error(response.Path("error.message").Data().(string))
			continue
		}

		// Insert the resource into the database
		_, err = resourceInsertPrepared.Exec(resourceId, subscriptionId, resourceGroup["id"].(string), listedResource["type"].(string), response.String())
		if err != nil {
			log.Errorf("Error inserting resource: %s", err)
		}
	}
	log.Infof("Finished enumerating %s : %s", subscriptionName, resourceGroup["name"].(string))
}

func enumerateArm(ctx *CollectorContext) {
	log.Info("Enumerating ARM resources")

	var err error
	var tenants []map[string]interface{}
	var subscriptions []map[string]interface{}

	// Create the tables
	tableStatements := []string{
		"CREATE TABLE IF NOT EXISTS tenants (id TEXT PRIMARY KEY, data BLOB)",
		"CREATE TABLE IF NOT EXISTS subscriptions (id TEXT PRIMARY KEY, data BLOB)",
		"CREATE TABLE IF NOT EXISTS resources (id TEXT PRIMARY KEY, sub_id TEXT, rg_id TEXT, resource_type TEXT, data BLOB)",
		"CREATE TABLE IF NOT EXISTS roleAssignments (name TEXT PRIMARY KEY, id TEXT, data BLOB)",
	}
	for _, statement := range tableStatements {
		if err := ctx.OutputDB.Exec(statement); err != nil {
			log.Error(err)
		}
	}

	tenants, err = QueryResources(ctx.ARMCred, ctx.CloudEndpoints.ArmUrl, "tenants?api-version=2022-12-01", nil)
	if err != nil && tenants == nil || len(tenants) == 0 {
		log.Errorf("Could not get tenants: %s", err)
		return
	}
	log.Infof("Tenants found: (%d)", len(tenants))

	if len(tenants) == 0 {
		log.Error("No tenants found")
		return
	}

	// Write the tenants to the output file then query the subscriptions
	// There may be multiple tenants if the user is a guest in other tenants
	// But the access token will only be valid for one tenant ("tid" claim)
	for _, tenant := range tenants {
		tenantData, err := json.Marshal(tenant)
		if err != nil {
			log.Error(err)
			continue
		}
		err = ctx.OutputDB.Exec("INSERT INTO tenants (id, data) VALUES (?, jsonb(?))", tenant["id"].(string), string(tenantData))
		if err != nil {
			log.Error(err)
		}
	}

	subscriptions, err = QueryResources(ctx.ARMCred, ctx.CloudEndpoints.ArmUrl, "/subscriptions/?api-version=2024-08-01", nil)
	if err != nil && subscriptions == nil || len(subscriptions) == 0 {
		log.Errorf("Could not get subscriptions: %s", err)
		return
	}

	log.Infof("Subscriptions found: (%d)", len(subscriptions))

	subscriptionWg := new(sync.WaitGroup)

	// This event is used to throttle the requests to the ARM API
	// to avoid hitting the rate limit. It is set to 100 requests per second.
	// This is a rough estimate and may need to be adjusted based on the API limits.
	throttleEvent := NewEvent()

	for _, subscription := range subscriptions {
		subscriptionData, err := json.Marshal(subscription)
		if err != nil {
			log.Error(err)
			continue
		}
		err = ctx.OutputDB.Exec("INSERT INTO subscriptions (id, data) VALUES (?, jsonb(?))", subscription["subscriptionId"].(string), string(subscriptionData))
		if err != nil {
			log.Error(err)
		}

		subscriptionWg.Add(1)
		go func(subscription map[string]interface{}) {
			defer subscriptionWg.Done()
			enumerateSubscription(ctx, subscription, throttleEvent)
		}(subscription)

	}
	subscriptionWg.Wait()
	log.Info("Finished enumerating ARM resources")
}
