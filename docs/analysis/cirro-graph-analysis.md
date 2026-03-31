# Cirro Graph Analysis

This document provides an overview of the analysis capabilities built into Cirro Graph for identifying security risks and misconfigurations in cloud environments. Currently focused on Azure environments, with the ability to expand to other platforms in the future. You can explore the help for these features by running `cirro graph -h`.

## DNS Security Analysis

The DNS security analysis module performs comprehensive checks on domain configurations within cloud identity providers (currently Entra ID) and application registrations to identify potential security vulnerabilities related to domain ownership and DNS misconfigurations.

### Purpose

The DNS analysis functionality helps security teams identify potential domain takeover risks and misconfigured domain settings that could be exploited by attackers. It focuses on domains referenced in cloud platform configurations (currently Entra ID) that may be vulnerable due to:

- Expired or unregistered domains
- Domains that resolve to non-existent DNS records (NXDOMAIN)
- Available domains that could be registered by malicious actors

### Key Analysis Areas

!!! warning "False Positives"
    Some Microsoft-owned domains (such as those ending in `.azureedge.net`, `.azurewebsites.net`, or other Azure-specific domains) may report as "available". These should be considered false positives and are not actually vulnerable to domain takeover attacks. Users should focus on third-party domains and custom organizational domains when evaluating results.

**Verified Domains**
- Analyzes organizational verified domains in Entra ID
- Checks both the primary domain and root domain for DNS resolution
- Identifies domains that may have expired or become available for re-registration

**Federated Identity Credentials**
- Examines issuer domains used in federated identity configurations
- Validates that identity provider domains are properly configured and owned
- Identifies potential federation hijacking opportunities

**Application Redirect URIs**
- Scans redirect URIs configured in Entra ID application registrations
- Checks for domains that may be vulnerable to takeover
- Helps identify potential OAuth flow hijacking risks

**Service Principal Login URLs**
- Analyzes login URLs configured for service principals
- Validates domain ownership for authentication endpoints
- Identifies potential authentication bypass opportunities

**Reply URLs**
- Examines reply URLs in service principal configurations
- Focuses on third-party applications and external organizations
- Helps identify potential token theft vulnerabilities

### Security Impact

This analysis helps organizations identify:
- **Domain Takeover Risks**: Domains that have expired or become available for registration
- **DNS Hijacking Opportunities**: Domains with NXDOMAIN responses that could be registered maliciously
- **Authentication Bypass Vectors**: Misconfigured URLs that could redirect authentication flows
- **Federation Attacks**: Vulnerable identity provider domains that could compromise SSO security

### Output and Reporting

The analysis generates detailed reports in both tabular format and CSV exports, providing security teams with actionable intelligence about domain-related risks in their Azure environment. Each analysis includes domain status, availability information, and potential security implications.