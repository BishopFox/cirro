use crate::collect::azure::credentials::common::{AuthCredential, Token};
use crate::errors::CirroError;
use async_trait::async_trait;
use chrono::TimeZone;
use log::debug;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ClientSecretCredential {
    // The token endpoint for the client secret credential
    pub token_endpoint: &'static str,
    /// The client ID for the client secret credential
    pub client_id: String,
    /// The client secret for the client secret credential
    pub client_secret: String,
    /// The tenant ID for the client secret credential
    pub tenant_id: String,
    /// The resource for the Azure CLI credential
    pub resource: &'static str,
    /// The token for the Azure CLI credential
    pub token: RwLock<Token>,
}

/// Implementation of the AuthCredential trait for ClientSecretCredential
#[async_trait]
impl AuthCredential for ClientSecretCredential {
    async fn get_token(&self) -> Result<Token, CirroError> {
        let guard = self.token.read().await;
        if !guard.is_expired_or_not_set() {
            return Ok(guard.clone());
        }
        drop(guard);

        debug!("Fetching new token for client secret credential");
        let url = format!("{}/{}/oauth2/token", self.token_endpoint, self.tenant_id);
        let resp = reqwest::Client::new()
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("resource", &self.resource),
            ])
            .send()
            .await
            .map_err(|e| CirroError::NetworkError(e.to_string()))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| CirroError::Unknown(format!("JSON parsing error: {}", e)))?;

        let new_token = Token {
            access_token: Arc::new(
                resp["access_token"]
                    .as_str()
                    .ok_or_else(|| CirroError::ParseError("missing access_token".into()))?
                    .to_string(),
            ),
            expires_on: resp["expires_on"]
                .as_i64()
                .and_then(|ts| chrono::Utc.timestamp_opt(ts, 0).single())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            refresh_token: None,
        };

        debug!("New token fetched successfully for client secret credential");
        let mut write_guard = self.token.write().await;
        *write_guard = new_token.clone();

        Ok(new_token)
    }
}
