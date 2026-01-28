# SAME_USER

Links different user accounts that represent the same physical person across various systems and platforms.

**Relationship:** `:SAME_USER`

**Direction:** Various (depends on the specific account types being linked)

## Description

This relationship is established through post-processing analysis that correlates user identities across different systems based on matching attributes such as login names, email addresses, or other identifying information. This allows for comprehensive identity correlation across multiple platforms and services.

## Examples

### TSUser → GraphUser
Links Tailscale users to their corresponding Entra ID user accounts when they can be matched by login name (case-insensitive comparison). This allows correlation between a user's identity in Tailscale and their corporate Entra ID identity.

## Use Cases

- Correlating user activities and access across multiple platforms
- Understanding the full scope of a user's access and privileges across different systems
- Identifying potential security risks when the same person has elevated privileges across multiple platforms
- Enabling comprehensive user access reviews and compliance auditing
- Detecting account anomalies or inconsistencies for the same individual