use crate::credentials::common::{AuthCredential, AuthError, Token};
use async_trait::async_trait;
use log::{debug, info};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AzureCliCredential {
    /// The tenant ID for the Azure CLI credential
    pub tenant_id: Option<String>,
    /// The resource for the Azure CLI credential
    pub resource: &'static str,
    /// The token for the Azure CLI credential
    pub token: RwLock<Token>,
}

/// Implementation of the AuthCredential trait for AzureCliCredential
#[async_trait]
impl AuthCredential for AzureCliCredential {
    async fn get_token(&self) -> Result<Token, AuthError> {
        let guard = self.token.read().await;
        if !guard.is_expired_or_not_set() {
            return Ok(guard.clone());
        }
        drop(guard);

        info!("Fetching new token for Azure CLI credential");

        let mut args = vec!["account", "get-access-token", "--resource", &self.resource];
        if let Some(t) = &self.tenant_id {
            args.push("--tenant");
            args.push(t);
        }

        debug!("Running az command: az {}", args.join(" "));
        let output = tokio::process::Command::new("az")
            .args(&args)
            .output()
            .await
            .map_err(|e| AuthError::ProcessError(e))?;

        if !output.status.success() {
            return Err(AuthError::AuthenticationFailed(format!(
                "az failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        debug!(
            "az command output: {}",
            String::from_utf8_lossy(&output.stdout)
        );

        let resp: serde_json::Value =
            serde_json::from_slice(&output.stdout).map_err(|e| AuthError::JsonError(e))?;

        let mut new_token = Token {
            access_token: Arc::new(
                resp["accessToken"]
                    .as_str()
                    .ok_or_else(|| AuthError::ParseError("missing accessToken".into()))?
                    .to_string(),
            ),
            expires_on: None,
            refresh_token: None,
        };
        new_token
            .set_expires_on_from_token()
            .map_err(|e| AuthError::ParseError(format!("Failed to set expires_on: {}", e)))?;
        debug!("New token fetched successfully for client secret credential");
        let mut write_guard = self.token.write().await;
        *write_guard = new_token.clone();

        Ok(new_token)
    }
}
