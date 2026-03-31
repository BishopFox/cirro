use crate::collect::azure::credentials::common::{AuthCredential, Token};
use crate::errors::CirroError;
use async_trait::async_trait;
use tokio::sync::RwLock;

pub struct AccessTokenCredential {
    /// The token for the Azure CLI credential
    pub token: RwLock<Token>,
}

/// Implementation of the AuthCredential trait for AccessTokenCredential
#[async_trait]
impl AuthCredential for AccessTokenCredential {
    async fn get_token(&self) -> Result<Token, CirroError> {
        let guard = self.token.read().await;
        if !guard.is_expired_or_not_set() {
            return Ok(guard.clone());
        }

        // We can't fetch a new token here, so we return an error
        drop(guard);
        return Err(CirroError::AuthenticationFailed(
            "Access token credential has expired".into(),
        ));
    }
}
