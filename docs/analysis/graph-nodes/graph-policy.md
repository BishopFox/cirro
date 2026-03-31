# GraphPolicy

Represents Entra ID policies collected from Microsoft Graph. These policies define organizational settings and rules that govern user behavior and access within the Entra ID tenant.

**Labels:** `:GraphPolicy`

**Properties:**

- `id` - Policy ID (primary key, uses unique tenant_policy format)
- `displayName` - Policy display name  
- `type` - Policy type (e.g., "authorizationpolicy")
- `allowEmailVerifiedUsersToJoinOrganization` - Whether email-verified users can join the organization
- `allowInvitesFrom` - Who can send invitations (anyone, adminsAndGuestInviters, etc.)
- `allowUserConsentForRiskyApps` - Whether users can consent to risky applications
- `allowedToSignUpEmailBasedSubscriptions` - Whether users can sign up for email-based subscriptions
- `allowedToUseSSPR` - Whether users can use self-service password reset
- `blockMsolPowerShell` - Whether MSOnline PowerShell is blocked
- `allowedToCreateApps` - Whether users can create applications
- `allowedToCreateSecurityGroups` - Whether users can create security groups
- `allowedToCreateTenants` - Whether users can create tenants
- `allowedToReadBitlockerKeysForOwnedDevice` - Whether users can read BitLocker keys for owned devices
- `allowedToReadOtherUsers` - Whether users can read other users' information


**Notes:**
- Policy IDs are constructed as `{tenantId}_{policyType}` to ensure uniqueness
- Currently focuses on Authorization Policy type but can be extended for other policy types
- Policies are automatically linked to their parent organization via HAS_POLICY relationships
