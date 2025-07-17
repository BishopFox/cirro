# Cirro

This tool is sometimes in development. It's really cool though.

`go build .\collector` - Collects the information.
- If using access-token authentication, make sure you have the right audience for `graph.microsoft.com` or `management.azure.com` and limit the `--mode` to `msgraph` or `arm` respectively.
- 
`go build .\ingestor\` - Ingests into Neo4j.