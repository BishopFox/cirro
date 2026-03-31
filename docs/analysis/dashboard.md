# CirroDash Dashboard

[CirroDash](https://github.com/bishopfox/cirrodash) is a low-code dashboard builder that allows you to create interactive dashboards to visualize and analyze your Cirro graph data. The Cirro project includes a pre-configured CirroDash dashboard with common security analysis queries for cloud environments.

## Dashboard Overview

CirroDash provides multiple pages focused on different aspects of cloud security analysis (currently Azure and Entra ID). It is launched by a Rust binary, which means that the dashboard can include backend logic to perform analysis that can't be performed in the web application itself. 


## Extending the Dashboard

### Adding New Cards

1. **Identify security use cases** requiring visualization
2. **Develop and test queries** in Neo4j Browser first
3. **Create new dashboard reports** with appropriate visualizations
4. **Document query logic** and expected results

