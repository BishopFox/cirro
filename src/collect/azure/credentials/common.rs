use crate::errors::CirroError;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::sync::Arc;

// Using async_trait for better performance with static dispatch when possible
use async_trait::async_trait;

#[async_trait]
pub trait AuthCredential {
    /// Returns the access token for the credential
    async fn get_token(&self) -> Result<Token, CirroError>;
}

#[derive(Debug, Default, Clone)]
pub struct Token {
    pub access_token: Arc<String>,
    pub expires_on: Option<DateTime<Utc>>,
    pub refresh_token: Option<Arc<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: u64,
    pub oid: Option<String>,
    pub tid: Option<String>,
}

impl Token {
    /// Checks if the token is expired or not set
    /// Returns true if the token is empty, has no expiration time,
    /// or the expiration time is less than 15 seconds in the future
    pub fn is_expired_or_not_set(&self) -> bool {
        // If the token is not set, consider it expired
        if self.access_token.is_empty() {
            log::debug!("Token is not set or empty");
            return true;
        }

        // If the token has an expiration time, check if it is in the past or about to expire
        match self.expires_on {
            Some(expiration) => {
                // Add 15 second buffer to avoid using tokens that are about to expire
                let buffer_time = chrono::Duration::seconds(15);
                let current_time_with_buffer = Utc::now() + buffer_time;

                if current_time_with_buffer >= expiration {
                    log::debug!("Token is expired or about to expire within 15 seconds");
                    true
                } else {
                    false
                }
            }
            None => {
                log::debug!("Token has no expiration time set");
                true
            }
        }
    }

    /// Gets claims from the JWT token
    /// Returns an error if the token is not a valid JWT or if claims cannot be parsed
    pub fn get_claims(&self) -> Result<Claims, CirroError> {
        let token_parts: Vec<&str> = self.access_token.split('.').collect();
        if token_parts.len() != 3 {
            return Err(CirroError::AuthError("Invalid JWT token format".into()));
        }

        let claims_json = base64::engine::general_purpose::STANDARD_NO_PAD
            .decode(token_parts[1])
            .map_err(|e| CirroError::AuthError(format!("Failed to decode token: {}", e)))?;

        let claims: Claims = serde_json::from_slice(&claims_json)?;
        Ok(claims)
    }

    /// Sets the expiration time from the token claims
    /// Returns an error if the claims cannot be parsed or if the expiration time is invalid
    pub fn set_expires_on_from_token(&mut self) -> Result<(), CirroError> {
        let claims = self.get_claims()?;
        let expiration_time = DateTime::<Utc>::from_timestamp(claims.exp as i64, 0)
            .ok_or_else(|| CirroError::AuthError("Invalid expiration time in token".into()))?;
        self.expires_on = Some(expiration_time);
        Ok(())
    }
}
