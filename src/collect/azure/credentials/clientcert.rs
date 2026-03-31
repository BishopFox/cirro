use crate::collect::azure::credentials::common::{AuthCredential, Token};
use crate::errors::CirroError;
use async_trait::async_trait;
use base64::Engine;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use log::debug;
use pem::parse_many;
use sha1::{Digest, Sha1};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

pub struct ClientCertificateCredential {
    // The token endpoint for the client secret credential
    pub token_endpoint: &'static str,
    /// The client ID for the client secret credential
    pub client_id: String,
    /// The path to the client certificate
    pub certificate_path: PathBuf,
    /// The tenant ID for the client secret credential
    pub tenant_id: String,
    /// The resource for the Azure CLI credential
    pub resource: &'static str,
    /// The token for the Azure CLI credential
    pub token: RwLock<Token>,
}

// Define the claims structure outside any impl block
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ClientAssertionClaims {
    aud: String, // Audience - token endpoint URL
    exp: u64,    // Expiration time (seconds since UNIX epoch)
    iat: u64,    // Issued at time (seconds since UNIX epoch)
    iss: String, // Issuer - client ID
    jti: String, // JWT ID - unique identifier for this token
    nbf: u64,    // Not before time (seconds since UNIX epoch)
    sub: String, // Subject - same as issuer/client ID
}

impl ClientCertificateCredential {
    /// Generates a client assertion JWT for certificate-based authentication
    pub fn generate_client_assertion(&self) -> Result<String, CirroError> {
        // Read the certificate file
        let cert_data = fs::read_to_string(&self.certificate_path)
            .map_err(|e| CirroError::AuthError(e.to_string()))?;

        // Parse the PEM data to get multiple blocks (certificate and private key)
        let pem_blocks = parse_many(&cert_data)
            .map_err(|e| CirroError::AuthError(format!("Failed to parse PEM blocks: {}", e)))?;

        if pem_blocks.is_empty() {
            return Err(CirroError::AuthError(
                "No PEM blocks found in certificate file".into(),
            ));
        }

        debug!("Found {} PEM blocks in certificate file", pem_blocks.len());

        // Extract certificates and private keys from blocks
        let mut private_key = None;
        let mut certificates = Vec::new();

        for block in &pem_blocks {
            let tag = block.tag();
            if tag.contains("PRIVATE KEY") {
                debug!("Found private key block: {}", tag);
                private_key = Some(block);
            } else if tag.contains("CERTIFICATE") {
                debug!("Found certificate block: {}", tag);
                certificates.push(block);
            }
        }

        // Get the private key or return an error
        let private_key = private_key.ok_or_else(|| {
            CirroError::AuthError("No private key found in certificate file".into())
        })?;

        // Create JWT header with RS256 algorithm
        let mut header = Header::new(Algorithm::RS256);
        if let Some(cert) = certificates.first() {
            let mut thumbprint = Sha1::new();
            thumbprint.update(cert.contents());
            header.x5t = Some(
                base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .encode(thumbprint.finalize().to_vec()),
            );
        } else {
            debug!("No certificates found in the PEM blocks");
        }
        debug!("JWT header created: {:?}", header);

        // Generate a unique ID for the JWT
        let jti = uuid::Uuid::new_v4().to_string();

        // Calculate timestamps
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CirroError::AuthError(e.to_string()))?
            .as_secs();

        // Construct the audience (token endpoint with tenant)
        let audience = format!("{}/{}/oauth2/token", self.token_endpoint, self.tenant_id);

        // Create the claims
        let claims = ClientAssertionClaims {
            aud: audience,
            exp: now + 600, // 10 minutes
            iat: now,       // Issued at current time
            iss: self.client_id.clone(),
            jti,
            nbf: now,
            sub: self.client_id.clone(),
        };
        // Create the encoding key from the PEM private key
        let encoding_key = EncodingKey::from_rsa_pem(private_key.to_string().as_bytes())
            .map_err(|e| CirroError::AuthError(format!("Invalid RSA key: {}", e)))?;

        // Encode the JWT
        let token = encode(&header, &claims, &encoding_key)
            .map_err(|e| CirroError::AuthError(format!("JWT encoding error: {}", e)))?;
        Ok(token)
    }
}

/// Implementation of the AuthCredential trait for ClientCertificateCredential
#[async_trait]
impl AuthCredential for ClientCertificateCredential {
    async fn get_token(&self) -> Result<Token, CirroError> {
        let guard = self.token.read().await;
        if !guard.is_expired_or_not_set() {
            return Ok(guard.clone());
        }
        drop(guard);

        debug!("Fetching new token for client certificate credential");

        // Generate the client assertion JWT
        let client_assertion = self.generate_client_assertion()?;

        let url = format!("{}/{}/oauth2/token", self.token_endpoint, self.tenant_id);

        // Use client_assertion_type and client_assertion instead of client_secret
        let resp = reqwest::Client::new()
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.client_id),
                (
                    "client_assertion_type",
                    "urn:ietf:params:oauth:client-assertion-type:jwt-bearer",
                ),
                ("client_assertion", &client_assertion),
                ("resource", self.resource),
            ])
            .send()
            .await
            .map_err(|e| CirroError::AuthError(e.to_string()))?;

        // If the response is not successful, log the response body for debugging
        if !resp.status().is_success() {
            debug!("Failed to fetch token, status: {}", resp.status());
            let body = resp
                .text()
                .await
                .map_err(|e| CirroError::AuthError(e.to_string()))?;
            return Err(CirroError::AuthError(format!(
                "Failed to fetch token: {}",
                body
            )));
        }

        let resp = resp
            .json::<serde_json::Value>()
            .await
            .map_err(|e| CirroError::AuthError(format!("JSON parsing error: {}", e)))?;

        // Check for errors in the response
        if let Some(error) = resp.get("error") {
            let error_desc = resp
                .get("error_description")
                .and_then(|d| d.as_str())
                .unwrap_or("Unknown error");

            return Err(CirroError::AuthError(format!(
                "{}: {}",
                error.as_str().unwrap_or("error"),
                error_desc
            )));
        }
        // Process the successful response
        let mut new_token = Token {
            access_token: Arc::new(
                resp["access_token"]
                    .as_str()
                    .ok_or_else(|| CirroError::AuthError("missing access_token".into()))?
                    .to_string(),
            ),
            expires_on: None,
            refresh_token: None,
        };
        new_token
            .set_expires_on_from_token()
            .map_err(|e| CirroError::AuthError(format!("Failed to set expires_on: {}", e)))?;

        debug!("New token fetched successfully for client certificate credential");
        let mut write_guard = self.token.write().await;
        *write_guard = new_token.clone();

        Ok(new_token)
    }
}
