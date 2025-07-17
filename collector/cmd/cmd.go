package cmd

import (
	"context"
	"fmt"
	"os"

	"github.com/bishopfox/cirro/collector/collectors"
	log "github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

func AppHelpTemplate() string {
	return `NAME:
	{{.Name}} - {{.Usage}}
	{{if .Commands}}
COMMANDS:
{{range .Commands}}{{if not .HideHelp}}   {{join .Names ", "}}{{ "\t"}}{{.Usage}}{{ "\n" }}{{end}}{{end}}{{end}}{{if .VisibleFlags}}
GLOBAL OPTIONS:
	{{range .VisibleFlags}}{{.}}
	{{end}}{{end}}`
}

var ctx collectors.CollectorContext = collectors.DefaultCollectorContext()

func Run() {
	cli.AppHelpTemplate = AppHelpTemplate()

	log.SetFormatter(&log.TextFormatter{
		ForceColors:     true,
		FullTimestamp:   true,
		TimestampFormat: "2006-01-02 15:04:05",
	})

	commonAuthenticationFlags := []cli.Flag{
		&cli.StringFlag{
			Name:  "cloud",
			Usage: "Cloud to enumerate (public, china, german, usgov)",
			Value: "public",
			Action: func(c *cli.Context, flag string) error {
				cloud, err := collectors.ParseCloudFromString(flag)
				if err != nil {
					return err
				}
				ctx.Cloud = cloud
				ctx.CloudEndpoints = cloud.GetEndpoints()
				return nil
			},
		},
		&cli.BoolFlag{
			Name:  "debug",
			Usage: "Enable debug logging",
			Action: func(c *cli.Context, value bool) error {
				log.SetLevel(log.DebugLevel)
				log.SetReportCaller(true)
				return nil
			},
		},
	}

	commonCommandFlags := []cli.Flag{
		&cli.BoolFlag{
			Name:  "enrich-vault-certs",
			Usage: "Dumps accessible key vault certificates and keys (NOISY!)",
			Action: func(c *cli.Context, value bool) error {
				ctx.ContextOptions.EnrichVaultCerts = value
				return nil
			},
		},
		&cli.BoolFlag{
			Name:  "kv-elevate",
			Usage: "Used with --enrich-vault-certs to elevate permissions on the Key Vault if need (NOISY!)",
			Action: func(c *cli.Context, value bool) error {
				ctx.ContextOptions.KeyvaultElevate = value
				return nil
			},
		},
		&cli.BoolFlag{
			Name:  "enrich-caps",
			Usage: "Dumps conditional access policies",
			Action: func(c *cli.Context, value bool) error {
				ctx.ContextOptions.EnrichConditionalAccessPolicies = value
				return nil
			},
		},
	}

	authenticationSubcommands := []*cli.Command{
		{
			Name:  "access-token",
			Usage: "Authenticate with an access token",
			Flags: append([]cli.Flag{&cli.StringFlag{
				Name:        "token",
				Aliases:     []string{"t"},
				Usage:       "Access token",
				Required:    true,
				Destination: &ctx.RawToken,
			}}, commonAuthenticationFlags...),
			Before: func(c *cli.Context) error {
				ctx.AuthMethod = collectors.AccessToken
				return nil
			},
			Action: runCollector,
		},
		{
			Name:  "azcli",
			Usage: "Authenticate with Azure CLI",
			Flags: append([]cli.Flag{&cli.StringFlag{
				Name:        "tenant",
				Aliases:     []string{"t"},
				Usage:       "Tenant ID",
				Required:    false,
				Destination: &ctx.TenantID,
			}}, commonAuthenticationFlags...),
			Before: func(c *cli.Context) error {
				ctx.AuthMethod = collectors.AzCliAuth
				return nil
			},
			Action: runCollector,
		},
		{
			Name:  "client",
			Usage: "Authenticate with client secret",
			Flags: append([]cli.Flag{&cli.StringFlag{
				Name:        "tenant",
				Aliases:     []string{"t"},
				Usage:       "Tenant ID",
				Required:    true,
				Destination: &ctx.TenantID,
			}, &cli.StringFlag{
				Name:        "clientId",
				Aliases:     []string{"c"},
				Usage:       "Client ID",
				Required:    true,
				Destination: &ctx.ClientId,
			}, &cli.StringFlag{
				Name:        "clientSecret",
				Aliases:     []string{"s"},
				Usage:       "Client secret",
				Required:    true,
				Destination: &ctx.ClientSecret,
			},
			}, commonAuthenticationFlags...),
			Before: func(c *cli.Context) error {
				ctx.AuthMethod = collectors.ClientSecret
				return nil
			},
			Action: runCollector,
		},
		{
			Name:  "clientcert",
			Usage: "Authenticate with client certificate",
			Flags: append([]cli.Flag{&cli.StringFlag{
				Name:        "tenant",
				Aliases:     []string{"t"},
				Usage:       "Tenant ID [Required]",
				Required:    true,
				Destination: &ctx.TenantID,
			}, &cli.StringFlag{
				Name:        "clientId",
				Aliases:     []string{"u"},
				Usage:       "Client ID [Required]",
				Required:    true,
				Destination: &ctx.ClientId,
			}, &cli.PathFlag{
				Name:        "certificatePath",
				Aliases:     []string{"p"},
				Usage:       "Path to certificate file [Required]",
				Required:    true,
				Destination: &ctx.CertificatePath,
				TakesFile:   true,
			},
			}, commonAuthenticationFlags...),
			Before: func(c *cli.Context) error {
				ctx.AuthMethod = collectors.ClientCert
				return nil
			},
			Action: runCollector,
		},
	}

	app := &cli.App{
		Name:                 "cirro",
		Usage:                "Azure and Microsoft Graph enumeration",
		EnableBashCompletion: false,
		HideHelpCommand:      true,
		Suggest:              true,
		Commands: []*cli.Command{
			{
				Name:  "collect",
				Usage: "Collect data from Azure and Microsoft Graph",
				Flags: append([]cli.Flag{&cli.PathFlag{
					Name:      "output",
					Aliases:   []string{"o"},
					Usage:     "Output filename",
					TakesFile: true,
					Action: func(c *cli.Context, flag string) error {
						if flag != "" {
							ctx.DBPath = flag
						}
						return nil
					},
				},
					&cli.StringFlag{
						Name:  "mode",
						Usage: "Enumeration mode (msgraph, arm, both)",
						Value: "both",
						Action: func(c *cli.Context, flag string) error {
							mode, err := collectors.ParseEnumModeFromString(flag)
							if err != nil {
								return err
							}
							ctx.EnumMode = mode
							return nil
						},
					},
				}, commonCommandFlags...),
				Before:      setParentCommandName,
				Subcommands: authenticationSubcommands,
			},
			{
				Name:  "enrich",
				Usage: "Enrich collected data in the database without re-enumerating",
				Flags: append([]cli.Flag{&cli.PathFlag{
					Name:      "input",
					Aliases:   []string{"i"},
					Usage:     "Path to cirro SQLite database",
					Required:  true,
					TakesFile: true,
					Action: func(c *cli.Context, flag string) error {
						ctx.DBPath = flag
						return nil
					},
				}}, commonCommandFlags...),
				Before:      setParentCommandName,
				Subcommands: authenticationSubcommands,
			},
		},
	}

	if err := app.Run(os.Args); err != nil {
		println(fmt.Sprintf("\nERROR: %s", err))
	}
}

type parentCommand string

const parentCommandNameKey parentCommand = "ParentCommandName"

func setParentCommandName(c *cli.Context) error {
	c.Context = context.WithValue(c.Context, parentCommandNameKey, c.Command.Name)
	return nil
}

func runCollector(c *cli.Context) error {
	// Print parent command name from context
	if parentCommandName, ok := c.Context.Value(parentCommandNameKey).(string); ok {
		switch parentCommandName {
		case "collect":
			if err := collectors.RunCollector(c, &ctx, false); err != nil {
				return err
			}
		case "enrich":
			if err := collectors.RunCollector(c, &ctx, true); err != nil {
				return err
			}
		default:
			return fmt.Errorf("unknown parent command: %s", parentCommandName)
		}
	} else {
		return fmt.Errorf("parent command name not found in context")
	}
	return nil
}
