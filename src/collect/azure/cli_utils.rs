use once_cell::sync::Lazy;
use regex::Regex;
use uuid::Uuid;

static DOMAIN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,}$")
        .expect("valid domain regex")
});

/// Validates that the input is a valid UUID.
pub fn validate_uuid(input: &str) -> Result<String, String> {
    let _ = Uuid::parse_str(input).map_err(|_| "must be a valid UUID".to_string())?;
    Ok(input.to_string())
}

/// Validates that the input is either a valid UUID or a valid domain name.
pub fn validate_tenant_id(input: &str) -> Result<String, String> {
    if validate_uuid(input).is_ok() || DOMAIN_RE.is_match(input) {
        Ok(input.to_string())
    } else {
        Err("must be a valid UUID or domain".into())
    }
}
