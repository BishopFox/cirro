# EmailServiceDomain

Represents Azure Communication Services email domains.

**Labels:** `:ArmResource:EmailServiceDomain`

**Properties:**

- `id` - Email service domain resource ID (primary key)
- `dataLocation` - Data location
- `domainManagement` - Domain management type
- `fromSenderDomain` - From sender domain
- `mailFromSenderDomain` - Mail from sender domain
- `provisioningState` - Provisioning state
- `userEngagementTracking` - User engagement tracking setting
- `verificationRecords` - Domain verification records

**Relationships:**
- `HAS_DOMAIN` ← EmailService
