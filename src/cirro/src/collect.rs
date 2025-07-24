use crate::EnrichmentFlags;
use crate::collectors::arm;
use crate::collectors::graph;
use crate::context::CloudEndpoints;
use crate::context::CollectorContext;
use crate::credentials::azcli::AzureCliCredential;
use crate::credentials::common::AuthCredential;
use crate::credentials::common::AuthError;
use crate::db::{DBWriteMessage, SqliteDb};
use crate::errors::CirroError;
use crate::{AzureCloud, EnumerationMode};

use futures;
use log::{debug, error, info};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;

pub struct Collector {
    context: CollectorContext<Box<dyn AuthCredential + Send + Sync>>,
}

/// Implements Deref for Collector to allow direct access to CollectorContext
impl Deref for Collector {
    type Target = CollectorContext<Box<dyn AuthCredential + Send + Sync>>;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

/// Implements DerefMut for Collector to allow mutable access to CollectorContext
impl DerefMut for Collector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

/// Runs the collector, performing the actual data collection
async fn run_collector(mut collector: Collector) -> Result<(), CirroError> {
    info!("Using cloud: {:?}", collector.cloud);
    info!("Using mode: {:?}", collector.mode);
    info!(
        "Using output: {:?}",
        collector.output_path.to_path_buf().as_path()
    );

    // Create the database writer
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<DBWriteMessage>();
    let db_tx = Arc::new(tx);
    collector.db_writer = Some(db_tx.clone());

    let output_path = collector.output_path.clone();
    tokio::spawn(async move {
        let sqlite_db = SqliteDb::new(output_path);
        sqlite_db.run_writer(rx).await;
    });

    let graph_collector = Arc::new(collector);
    let arm_collector: Arc<Collector> = Arc::clone(&graph_collector);

    // Prepare async tasks for concurrent execution
    let mut tasks = Vec::new();

    // Configure and prepare Graph API enumeration task
    if graph_collector.mode == EnumerationMode::Graph
        || graph_collector.mode == EnumerationMode::Both
    {
        debug!("MS Graph enumeration mode is enabled");
        // Pre-fetch token to validate credentials
        if let Err(e) = graph_collector.msgraph_credential.get_token().await {
            return Err(CirroError::AuthError(e));
        } else {
            debug!("MS Graph token retrieved successfully");
        }

        // Create a task for Graph enumeration
        let graph_collector_clone = Arc::clone(&graph_collector);
        let graph_task = tokio::spawn(async move {
            debug!("Starting Graph API enumeration task");
            if let Err(e) = graph::enumerate_graph(graph_collector_clone).await {
                error!("Error collecting Graph data: {}", e);
            }
            debug!("Graph API enumeration task completed");
        });

        tasks.push(graph_task);
    }

    // Configure and prepare ARM API enumeration task
    if arm_collector.mode == EnumerationMode::Arm || arm_collector.mode == EnumerationMode::Both {
        debug!("ARM enumeration mode is enabled");

        // Pre-fetch token to validate credentials
        if let Err(e) = arm_collector.arm_credential.get_token().await {
            return Err(CirroError::AuthError(e));
        } else {
            debug!("ARM token retrieved successfully");
        }

        // Create a task for ARM enumeration
        let arm_collector_clone = Arc::clone(&arm_collector);
        let arm_task = tokio::spawn(async move {
            debug!("Starting ARM API enumeration task");
            if let Err(e) = arm::enumerate_arm(arm_collector_clone).await {
                error!("Error collecting ARM data: {}", e);
            }
            debug!("ARM API enumeration task completed");
        });

        tasks.push(arm_task);
    }

    // Wait for all tasks to complete
    if !tasks.is_empty() {
        debug!("Waiting for {} enumeration tasks to complete", tasks.len());
        futures::future::join_all(tasks).await;
        info!("All enumeration tasks completed");
    }

    // Wait for the DB writer to finish
    info!("Sending shutdown message to DB writer");
    db_tx.send(DBWriteMessage::Shutdown).unwrap();

    // Wait for the DB writer to process the shutdown message
    info!("Sleeping for 5 seconds to ensure DB writer processes shutdown");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    info!("Done");
    Ok(())
}

/// Runs the enrichment functions
async fn run_enrichment(mut collector: Collector) -> Result<(), CirroError> {
    info!("Using cloud: {:?}", collector.cloud);
    info!(
        "Using output: {:?}",
        collector.output_path.to_path_buf().file_name().unwrap()
    );

    // Create the database writer
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<DBWriteMessage>();
    let db_tx = Arc::new(tx);
    collector.db_writer = Some(db_tx.clone());

    let output_path = collector.output_path.clone();
    let output_path_for_db = output_path.clone();
    tokio::spawn(async move {
        let db_writer = SqliteDb::new(output_path_for_db);
        db_writer.run_writer(rx).await;
    });

    // Run the enrichment functions
    // Check each flag in enrich_flags and print if true
    // If a function fails, continue to the next one
    let sqlite_conn = SqliteDb::new(output_path).get_connection().await?;
    let enrich_flags = collector.enrich_flags.as_ref().unwrap();
    if enrich_flags.storage_keys {
        if let Err(e) = collector.enrich_storage_account_keys(sqlite_conn).await {
            error!("Error enriching storage blobs: {}", e);
        }
    }

    // Wait for the DB writer to finish
    info!("Sending shutdown message to DB writer");
    db_tx.send(DBWriteMessage::Shutdown).unwrap();

    // Wait for the DB writer to process the shutdown message
    info!("Sleeping for 5 seconds to ensure DB writer processes shutdown");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    info!("Done");
    Ok(())
}

/// Collects data using Access Token credentials
pub async fn collect_with_access_token(
    token: String,
    cloud: AzureCloud,
    output_path: PathBuf,
    enrich_mode: bool,
    enrich_flags: Option<EnrichmentFlags>,
) -> Result<(), CirroError> {
    // We don't really need to create all of these credentials for Access Token mode,
    // but we do it to keep the interface consistent with other authentication modes.
    // We can probably optimize this later.
    let mut arm_credential = Box::new(crate::credentials::access_token::AccessTokenCredential {
        token: tokio::sync::RwLock::new(crate::credentials::common::Token::default()),
    });
    let mut msgraph_credential =
        Box::new(crate::credentials::access_token::AccessTokenCredential {
            token: tokio::sync::RwLock::new(crate::credentials::common::Token::default()),
        });
    let vault_credential = Box::new(crate::credentials::access_token::AccessTokenCredential {
        token: tokio::sync::RwLock::new(crate::credentials::common::Token::default()),
    });

    // Get the token audience and compute the cloud endpoints
    // This is necessary to ensure the correct resource URLs are used for the token.
    let cloud_endpoints: CloudEndpoints = CloudEndpoints::new(cloud);
    let mut token = crate::credentials::common::Token {
        access_token: Arc::new(token),
        expires_on: None,
        refresh_token: None,
    };
    token.set_expires_on_from_token().map_err(|e| {
        CirroError::AuthError(AuthError::ParseError(format!(
            "Failed to set expires_on: {}",
            e
        )))
    })?;

    let credential = Box::new(crate::credentials::access_token::AccessTokenCredential {
        token: tokio::sync::RwLock::new(token.clone()),
    });

    // Get audience claim
    let audience = credential.get_token().await?.get_claims().unwrap().aud;
    let mode: EnumerationMode;

    info!("Using token with audience: {}", audience);
    match audience.as_str() {
        // Audience can sometimes be the App ID URI or the resource URL
        _ if audience == cloud_endpoints.arm_url
            || audience == "00000002-0000-0000-c000-000000000000" =>
        {
            mode = EnumerationMode::Arm;
            arm_credential = credential;
        }
        _ if audience == cloud_endpoints.msgraph_url
            || audience == "00000003-0000-0000-c000-000000000000" =>
        {
            mode = EnumerationMode::Graph;
            msgraph_credential = credential;
        }
        _ => {
            return Err(CirroError::AuthError(AuthError::Unknown(
                "Invalid token audience".into(),
            )));
        }
    }

    // Enrich mode is only supported in Arm mode
    if enrich_mode && mode != EnumerationMode::Arm {
        return Err(CirroError::AuthError(AuthError::Unknown(
            "Enrichment mode is only supported in Arm mode".into(),
        )));
    }

    info!("Starting with Access Token credentials");

    // Initialize the collector context with Access Token credentials
    let collector = Collector {
        context: CollectorContext::<Box<dyn AuthCredential + Send + Sync>> {
            mode,
            cloud,
            cloud_endpoints: CloudEndpoints::new(cloud),
            msgraph_credential: msgraph_credential as Box<dyn AuthCredential + Send + Sync>,
            arm_credential: arm_credential as Box<dyn AuthCredential + Send + Sync>,
            vault_credential: vault_credential as Box<dyn AuthCredential + Send + Sync>,
            output_path,
            tenant_id: None,
            subscription_id: None,
            client_id: None,
            client_secret: None,
            client_cert_path: None,
            db_writer: None,
            enrich_mode,
            enrich_flags,
        },
    };

    // Run the collector
    if collector.enrich_mode {
        if let Err(e) = run_enrichment(collector).await {
            error!("Error enriching data: {}", e);
            return Err(e);
        }
    } else {
        if let Err(e) = run_collector(collector).await {
            error!("Error collecting data: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// Collects data using Azure CLI credentials
pub async fn collect_with_azure_cli(
    tenant_id: Option<String>,
    subscription_id: Option<String>,
    mode: EnumerationMode,
    cloud: AzureCloud,
    output_path: PathBuf,
    enrich_mode: bool,
    enrich_flags: Option<EnrichmentFlags>,
) -> Result<(), CirroError> {
    info!("Starting with Azure CLI credentials");

    let cloud_endpoints = CloudEndpoints::new(cloud);

    // Initialize the Azure CLI credentials
    let arm_credential = Box::new(AzureCliCredential {
        tenant_id: tenant_id.clone(),
        resource: &cloud_endpoints.arm_url,
        token: tokio::sync::RwLock::new(crate::credentials::common::Token::default()),
    });
    let msgraph_credential = Box::new(AzureCliCredential {
        tenant_id: tenant_id.clone(),
        resource: &cloud_endpoints.msgraph_url,
        token: tokio::sync::RwLock::new(crate::credentials::common::Token::default()),
    });
    let vault_credential = Box::new(AzureCliCredential {
        tenant_id: tenant_id.clone(),
        resource: &cloud_endpoints.vault_url,
        token: tokio::sync::RwLock::new(crate::credentials::common::Token::default()),
    });

    // Initialize the collector context with Azure CLI credentials
    let collector = Collector {
        context: CollectorContext::<Box<dyn AuthCredential + Send + Sync>> {
            mode,
            cloud,
            cloud_endpoints: CloudEndpoints::new(cloud),
            msgraph_credential: msgraph_credential as Box<dyn AuthCredential + Send + Sync>,
            arm_credential: arm_credential as Box<dyn AuthCredential + Send + Sync>,
            vault_credential: vault_credential as Box<dyn AuthCredential + Send + Sync>,
            output_path: output_path,
            tenant_id: tenant_id.clone(),
            subscription_id,
            client_id: None,
            client_secret: None,
            client_cert_path: None,
            db_writer: None,
            enrich_mode,
            enrich_flags,
        },
    };

    // Run the collector
    if collector.enrich_mode {
        if let Err(e) = run_enrichment(collector).await {
            error!("Error enriching data: {}", e);
            return Err(e);
        }
    } else {
        if let Err(e) = run_collector(collector).await {
            error!("Error collecting data: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// Collects data using Client Secret credentials
pub async fn collect_with_client_secret(
    tenant_id: String,
    client_id: String,
    client_secret: String,
    mode: EnumerationMode,
    cloud: AzureCloud,
    output_path: PathBuf,
    enrich_mode: bool,
    enrich_flags: Option<EnrichmentFlags>,
) -> Result<(), CirroError> {
    info!("Starting with Client Secret credentials");

    let cloud_endpoints = CloudEndpoints::new(cloud);

    // Initialize the Client Secret credentials
    let arm_credential = Box::new(crate::credentials::clientsecret::ClientSecretCredential {
        token_endpoint: &cloud_endpoints.token_endpoint,
        client_id: client_id.clone(),
        client_secret: client_secret.clone(),
        tenant_id: tenant_id.clone(),
        resource: &cloud_endpoints.arm_url,
        token: tokio::sync::RwLock::new(crate::credentials::common::Token::default()),
    });
    let msgraph_credential = Box::new(crate::credentials::clientsecret::ClientSecretCredential {
        token_endpoint: &cloud_endpoints.token_endpoint,
        client_id: client_id.clone(),
        client_secret: client_secret.clone(),
        tenant_id: tenant_id.clone(),
        resource: cloud_endpoints.msgraph_url,
        token: tokio::sync::RwLock::new(crate::credentials::common::Token::default()),
    });
    let vault_credential = Box::new(crate::credentials::clientsecret::ClientSecretCredential {
        token_endpoint: &cloud_endpoints.token_endpoint,
        client_id: client_id.clone(),
        client_secret: client_secret.clone(),
        tenant_id: tenant_id.clone(),
        resource: cloud_endpoints.vault_url,
        token: tokio::sync::RwLock::new(crate::credentials::common::Token::default()),
    });

    // Initialize the collector context with Client Secret credentials
    let collector = Collector {
        context: CollectorContext::<Box<dyn AuthCredential + Send + Sync>> {
            mode,
            cloud,
            cloud_endpoints: CloudEndpoints::new(cloud),
            msgraph_credential: msgraph_credential as Box<dyn AuthCredential + Send + Sync>,
            arm_credential: arm_credential as Box<dyn AuthCredential + Send + Sync>,
            vault_credential: vault_credential as Box<dyn AuthCredential + Send + Sync>,
            output_path,
            tenant_id: Some(tenant_id.clone()),
            subscription_id: None,
            client_id: None,
            client_secret: None,
            client_cert_path: None,
            db_writer: None,
            enrich_mode,
            enrich_flags,
        },
    };

    // Run the collector
    if collector.enrich_mode {
        if let Err(e) = run_enrichment(collector).await {
            error!("Error enriching data: {}", e);
            return Err(e);
        }
    } else {
        if let Err(e) = run_collector(collector).await {
            error!("Error collecting data: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

/// Collects data using Client Certificate credentials
pub async fn collect_with_client_cert(
    tenant_id: String,
    client_id: String,
    client_certificate: PathBuf,
    mode: EnumerationMode,
    cloud: AzureCloud,
    output_path: PathBuf,
    enrich_mode: bool,
    enrich_flags: Option<EnrichmentFlags>,
) -> Result<(), CirroError> {
    info!("Starting with Client Certificate credentials");

    let cloud_endpoints = CloudEndpoints::new(cloud);

    // Initialize the Client Certificate credentials
    let arm_credential = Box::new(
        crate::credentials::clientcert::ClientCertificateCredential {
            token_endpoint: &cloud_endpoints.token_endpoint,
            client_id: client_id.clone(),
            certificate_path: client_certificate.clone(),
            tenant_id: tenant_id.clone(),
            resource: &cloud_endpoints.arm_url,
            token: tokio::sync::RwLock::new(crate::credentials::common::Token::default()),
        },
    );
    let msgraph_credential = Box::new(
        crate::credentials::clientcert::ClientCertificateCredential {
            token_endpoint: &cloud_endpoints.token_endpoint,
            client_id: client_id.clone(),
            certificate_path: client_certificate.clone(),
            tenant_id: tenant_id.clone(),
            resource: cloud_endpoints.msgraph_url,
            token: tokio::sync::RwLock::new(crate::credentials::common::Token::default()),
        },
    );
    let vault_credential = Box::new(
        crate::credentials::clientcert::ClientCertificateCredential {
            token_endpoint: &cloud_endpoints.token_endpoint,
            client_id: client_id.clone(),
            certificate_path: client_certificate.clone(),
            tenant_id: tenant_id.clone(),
            resource: cloud_endpoints.vault_url,
            token: tokio::sync::RwLock::new(crate::credentials::common::Token::default()),
        },
    );

    // Initialize the collector context with Client Certificate credentials
    let collector = Collector {
        context: CollectorContext::<Box<dyn AuthCredential + Send + Sync>> {
            mode,
            cloud,
            cloud_endpoints: CloudEndpoints::new(cloud),
            msgraph_credential: msgraph_credential as Box<dyn AuthCredential + Send + Sync>,
            arm_credential: arm_credential as Box<dyn AuthCredential + Send + Sync>,
            vault_credential: vault_credential as Box<dyn AuthCredential + Send + Sync>,
            output_path,
            tenant_id: Some(tenant_id.clone()),
            subscription_id: None,
            client_id: Some(client_id),
            client_secret: None,
            client_cert_path: Some(client_certificate),
            db_writer: None,
            enrich_mode,
            enrich_flags: enrich_flags,
        },
    };
    // Run the collector
    if collector.enrich_mode {
        if let Err(e) = run_enrichment(collector).await {
            error!("Error enriching data: {}", e);
            return Err(e);
        }
    } else {
        if let Err(e) = run_collector(collector).await {
            error!("Error collecting data: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
