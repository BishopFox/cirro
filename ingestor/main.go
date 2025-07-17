package main

import (
	"embed"
	"fmt"
	"os"

	"github.com/bishopfox/cirro/ingestor/core"
	log "github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

var assets embed.FS

// //go:embed build/appicon.png
// var icon []byte

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

func main() {
	cli.AppHelpTemplate = AppHelpTemplate()

	log.SetFormatter(&log.TextFormatter{
		ForceColors:     true,
		FullTimestamp:   true,
		TimestampFormat: "2006-01-02 15:04:05",
	})

	app := &cli.App{
		Name:                 "cirro-ingest",
		Usage:                "Cirro Ingestor",
		EnableBashCompletion: true,
		HideHelpCommand:      true,
		Suggest:              true,
		Commands: []*cli.Command{
			{
				Name:  "ingest",
				Usage: "Ingest a Cirro results sqlite db into Neo4J",
				Flags: []cli.Flag{
					&cli.PathFlag{
						Name:      "file",
						Aliases:   []string{"f"},
						Usage:     "SQLite file to ingest",
						TakesFile: true,
						Required:  true,
					},
					&cli.StringFlag{
						Name:    "dbuser",
						Aliases: []string{"u"},
						Usage:   "Neo4J user",
						Value:   "neo4j",
					},
					&cli.StringFlag{
						Name:    "dbpass",
						Aliases: []string{"p"},
						Usage:   "Neo4J password",
						Value:   "password",
					},
					&cli.StringFlag{
						Name:    "server",
						Aliases: []string{"s"},
						Usage:   "Neo4J server",
						Value:   "bolt://127.0.0.1:7687",
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
				},
				Action: func(c *cli.Context) error {
					if err := RunIngestor(c); err != nil {
						return err
					}
					return nil
				},
			},
		},
	}

	if err := app.Run(os.Args); err != nil {
		println(fmt.Sprintf("\nERROR: %s", err))
	}
}

func RunIngestor(c *cli.Context) error {
	ingestor, err := core.NewCirroIngestor(c.Path("file"), c.String("dbuser"), c.String("dbpass"), c.String("server"))
	if err != nil {
		return err
	}
	return ingestor.Run()
}
