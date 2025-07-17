package collectors

import (
	"errors"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/bishopfox/cirro/collector/credentials"
	"github.com/golang-jwt/jwt/v5"
	log "github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

func RunCollector(cli *cli.Context, ctx *CollectorContext, enrichOnly bool) error {
	var err error

	// Start the collector in the specified mode
	if !enrichOnly {
		log.Info("Starting collector in " + ctx.EnumMode.LongString() + " mode")
		log.Info("Attempting to login with auth mode - " + ctx.AuthMethod.LongString())

		// If auth mode is not set to ArmEnum or BothEnum, then disable vault enrichment
		if ctx.EnumMode != ArmEnum && ctx.EnumMode != BothEnum {
			log.Warn("Vault enrichment is disabled in non-ARM enumeration modes")
			ctx.ContextOptions.EnrichVaultCerts = false
		}
	}

	ctx.CloudEndpoints = ctx.Cloud.GetEndpoints()
	err = verifyAuthMode(ctx)
	if err != nil {
		return err
	}

	// Open the database
	ctx.OutputDB, err = NewCirroDB(ctx.DBPath)
	if err != nil {
		return err
	}
	defer ctx.OutputDB.Close()

	err = ctx.OutputDB.Init()
	if err != nil {
		log.Error(err)
		return err
	}

	if !enrichOnly {
		wg := new(sync.WaitGroup)
		start := time.Now()
		runningEnumeration := false

		if ctx.EnumMode == Graph || ctx.EnumMode == BothEnum {
			if token, err := ctx.MSGraphCred.GetToken(); token != nil {
				wg.Add(1)
				runningEnumeration = true
				// Enumerate Microsoft Graph
				go func() {
					defer wg.Done()
					enumerateGraph(ctx)
				}()
			} else {
				log.Error(err)
			}

		}

		if ctx.EnumMode == ArmEnum || ctx.EnumMode == BothEnum {
			if token, err := ctx.ARMCred.GetToken(); token != nil {
				wg.Add(1)
				runningEnumeration = true
				// Enumerate Azure Resource Manager
				go func() {
					defer wg.Done()
					enumerateArm(ctx)
				}()
			} else {
				log.Error(err)
			}
		}

		wg.Wait()
		if runningEnumeration {
			elapsed := time.Since(start)
			log.Infof("Finished enumeration (%s)", elapsed)
			log.Info("Results output to " + ctx.DBPath + ".db")
		}

		// Flush the database
		err = ctx.OutputDB.Commit()
		if err != nil {
			return err
		}
	}

	if enrichOnly {
		log.Infof("Starting enrichment using database: %s", ctx.DBPath)
	}

	if !ctx.ContextOptions.EnrichVaultCerts && !ctx.ContextOptions.EnrichConditionalAccessPolicies {
		log.Warn("No enrichments enabled. See --help for available enrichments flags")
	}

	// Enrichments go here
	if ctx.ContextOptions.EnrichVaultCerts {
		err = EnrichKeyVaultCertificates(ctx)
		if err != nil {
			return err
		}
	}

	if ctx.ContextOptions.EnrichConditionalAccessPolicies {
		err = EnrichConditionalAccessPolicies(ctx)
		if err != nil {
			return err
		}
	}

	return nil

}

func verifyAuthMode(ctx *CollectorContext) error {
	switch ctx.AuthMethod {
	case AccessToken:

		parsedToken, _, err := new(jwt.Parser).ParseUnverified(ctx.RawToken, jwt.MapClaims{})
		if err != nil {
			return err
		}

		expirationTime := time.Unix(int64(parsedToken.Claims.(jwt.MapClaims)["exp"].(float64)), 0)
		audience := parsedToken.Claims.(jwt.MapClaims)["aud"].(string)

		if strings.Contains(audience, ctx.CloudEndpoints.MsGraphUrl) {
			ctx.MSGraphCred = &credentials.AccessTokenCredential{
				Token: credentials.Token{
					AccessToken: ctx.RawToken,
					ExpiresOn:   expirationTime,
				},
			}
		} else if strings.Contains(audience, ctx.CloudEndpoints.ArmUrl) {

			ctx.ARMCred = &credentials.AccessTokenCredential{
				Token: credentials.Token{
					AccessToken: ctx.RawToken,
					ExpiresOn:   expirationTime,
				},
			}
		} else {
			return fmt.Errorf("token audience does not match the Graph/ARM URLs: %s not in (%s, %s)", audience, ctx.CloudEndpoints.MsGraphUrl, ctx.CloudEndpoints.ArmUrl)
		}
	case AzCliAuth:

		ctx.MSGraphCred = &credentials.AzureCliCredential{
			Resource: ctx.CloudEndpoints.MsGraphUrl,
		}
		ctx.ARMCred = &credentials.AzureCliCredential{
			Resource: ctx.CloudEndpoints.ArmUrl,
		}
		ctx.VaultCred = &credentials.AzureCliCredential{
			Resource: ctx.CloudEndpoints.VaultUrl,
		}
		ctx.AadGraphCred = &credentials.AzureCliCredential{
			Resource: ctx.CloudEndpoints.AadGraphUrl,
		}
	case ClientSecret:

		ctx.MSGraphCred = &credentials.ClientCredentialSecret{
			TokenEndpoint: ctx.CloudEndpoints.TokenEndpoint,
			Resource:      ctx.CloudEndpoints.MsGraphUrl,
			TenantID:      ctx.TenantID,
			ClientId:      ctx.ClientId,
			ClientSecret:  ctx.ClientSecret,
		}
		ctx.ARMCred = &credentials.ClientCredentialSecret{
			TokenEndpoint: ctx.CloudEndpoints.TokenEndpoint,
			Resource:      ctx.CloudEndpoints.ArmUrl,
			TenantID:      ctx.TenantID,
			ClientId:      ctx.ClientId,
			ClientSecret:  ctx.ClientSecret,
		}
		ctx.VaultCred = &credentials.ClientCredentialSecret{
			TokenEndpoint: ctx.CloudEndpoints.TokenEndpoint,
			Resource:      ctx.CloudEndpoints.VaultUrl,
			TenantID:      ctx.TenantID,
			ClientId:      ctx.ClientId,
			ClientSecret:  ctx.ClientSecret,
		}
		ctx.AadGraphCred = &credentials.ClientCredentialSecret{
			TokenEndpoint: ctx.CloudEndpoints.TokenEndpoint,
			Resource:      ctx.CloudEndpoints.AadGraphUrl,
			TenantID:      ctx.TenantID,
			ClientId:      ctx.ClientId,
			ClientSecret:  ctx.ClientSecret,
		}
	case ClientCert:

		ctx.MSGraphCred = &credentials.ClientCredentialCert{
			TokenEndpoint:   ctx.CloudEndpoints.TokenEndpoint,
			Resource:        ctx.CloudEndpoints.MsGraphUrl,
			TenantID:        ctx.TenantID,
			ClientId:        ctx.ClientId,
			CertificatePath: ctx.CertificatePath,
		}
		ctx.ARMCred = &credentials.ClientCredentialCert{
			TokenEndpoint:   ctx.CloudEndpoints.TokenEndpoint,
			Resource:        ctx.CloudEndpoints.ArmUrl,
			TenantID:        ctx.TenantID,
			ClientId:        ctx.ClientId,
			CertificatePath: ctx.CertificatePath,
		}
		ctx.VaultCred = &credentials.ClientCredentialCert{
			TokenEndpoint:   ctx.CloudEndpoints.TokenEndpoint,
			Resource:        ctx.CloudEndpoints.VaultUrl,
			TenantID:        ctx.TenantID,
			ClientId:        ctx.ClientId,
			CertificatePath: ctx.CertificatePath,
		}
		ctx.AadGraphCred = &credentials.ClientCredentialCert{
			TokenEndpoint:   ctx.CloudEndpoints.TokenEndpoint,
			Resource:        ctx.CloudEndpoints.AadGraphUrl,
			TenantID:        ctx.TenantID,
			ClientId:        ctx.ClientId,
			CertificatePath: ctx.CertificatePath,
		}
	default:
		return errors.New("unsupported authentication method")
	}
	return nil
}
