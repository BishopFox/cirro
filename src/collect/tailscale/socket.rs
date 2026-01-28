use crate::errors::CirroError;
use log::info;
use reqwest::ClientBuilder;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug)]
pub struct TSSocketCollector {
    output_path: PathBuf,
    client: reqwest::Client,
}

impl TSSocketCollector {
    /// Create a new Tailscale Windows Socket Collector using the specified named pipe path
    pub async fn new(socket_name: PathBuf, output_path: PathBuf) -> Result<Self, CirroError> {
        // Create a named pipe client

        info!(
            "Connecting to Tailscale named pipe at {}",
            socket_name.display()
        );

        #[cfg(target_os = "windows")]
        let client = ClientBuilder::new()
            .windows_named_pipe(socket_name.to_string_lossy().as_ref())
            .build()
            .map_err(|e| CirroError::Unknown(e.to_string()))?;

        #[cfg(not(target_os = "windows"))]
        let client = ClientBuilder::new()
            .unix_socket(socket_name.to_string_lossy().as_ref())
            .build()
            .map_err(|e| CirroError::Unknown(e.to_string()))?;

        Ok(TSSocketCollector {
            client,
            output_path,
        })
    }

    pub async fn run(&self) -> Result<(), CirroError> {
        // Make a request to the Tailscale named pipe
        let response = self
            .client
            .get("http://localhost/localapi/v0/status")
            .header("Host", "local-tailscaled.sock")
            .header("Accept", "application/json")
            .header("Connection", "close")
            .send()
            .await?;

        let status = response.status();
        if status != reqwest::StatusCode::OK {
            return Err(CirroError::HttpError(format!(
                "Failed to connect to Tailscale named pipe: HTTP {}",
                status
            )));
        }

        let body = response.text().await?;
        info!("Received response from Tailscale named pipe: {}", body);

        // Write output to file
        fs::write(&self.output_path, body)
            .await
            .map_err(|e| CirroError::IoError(e))?;

        Ok(())
    }
}
