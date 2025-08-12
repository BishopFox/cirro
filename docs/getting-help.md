# Getting Help

<div class="card" markdown>
**:material-github: GitHub Issues**

If you can't find a solution in the documentation:

- [Bug Reports](https://github.com/bishopfox/cirro/issues)
- [Feature Requests](https://github.com/bishopfox/cirro/issues)
- [Discussions](https://github.com/bishopfox/cirro/discussions)
</div>

## Troubleshooting

### Common Issues

**Authentication Problems**

- Verify your Azure credentials are valid
- Check that your account has appropriate permissions
- Ensure you're targeting the correct Azure cloud environment

**Database Connection Issues**

- Verify your graph database is running and accessible
- Check connection strings and credentials
- Ensure firewall rules allow connections

**Data Collection Problems**

- Review debug logs with the `--debug` flag
- HTTP 429 responses might occur in large environments. Retry logic is built into Cirro.
- Verify permissions for target resources

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
# Collection with debug output
cirro collect azcli --debug

# Ingestion with debug output
cirro-graph --file cirro_output.db --debug
```