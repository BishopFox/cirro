use crate::collect::azure::cli::{AzureCloud, EnumerationMode, OptionEnumFlags};
use crate::collect::azure::collectors::arm;
use crate::collect::azure::collectors::graph;
use crate::collect::azure::context::CloudEndpoints;
use crate::collect::azure::context::CollectorContext;
use crate::collect::azure::credentials::azcli::AzureCliCredential;
use crate::collect::azure::credentials::common::AuthCredential;
use crate::collect::azure::db::{DBWriteMessage, SqliteDb};
use crate::errors::CirroError;

use futures;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;

pub struct Collector {
    context: CollectorContext<Box<dyn AuthCredential + Send + Sync + 'static>>,
}

/// Implements Deref for Collector to allow direct access to CollectorContext
impl Deref for Collector {
    type Target = CollectorContext<Box<dyn AuthCredential + Send + Sync + 'static>>;
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
async fn run_collector(mut collector: Collector, new_state: bool) -> Result<(), CirroError> {
    info!("Using cloud: {:?}", collector.cloud);
    info!("Using mode: {:?}", collector.mode);
    info!(
        "Using output: {:?}",
        collector.output_path.to_path_buf().as_path()
    );

    // Create the database writer with bounded channel for backpressure
    const DB_CHANNEL_CAPACITY: usize = 16_384;
    let (tx, rx) = tokio::sync::mpsc::channel::<DBWriteMessage>(DB_CHANNEL_CAPACITY);
    let db_tx = Arc::new(tx);
    collector.db_writer = Some(db_tx.clone());

    let output_path = collector.output_path.clone();
    let db_writer_task = tokio::task::spawn_blocking(move || {
        let sqlite_db = SqliteDb::new(output_path);
        sqlite_db.run_writer(rx);
    });

    // Read existing enumeration state (for resumable enumeration)
    let (graph_state, arm_state) = {
        let db = SqliteDb::new(collector.output_path.clone());
        if new_state {
            info!("--new-state: clearing saved enumeration state");
            db.clear_state(None)
                .map_err(|e| CirroError::DatabaseError(format!("Failed to clear state: {}", e)))?;
            (HashMap::new(), HashMap::new())
        } else {
            let gs = match db.read_state(None) {
                Ok(all_state) => {
                    // Split state into graph (no prefix) and arm (arm: prefix) entries
                    let graph: HashMap<_, _> = all_state
                        .iter()
                        .filter(|(k, _)| !k.starts_with("arm:"))
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    if !graph.is_empty() {
                        info!(
                            "Resuming graph enumeration for {} resource type(s)",
                            graph.len()
                        );
                        for (rt, (_, count, elapsed)) in &graph {
                            debug!(
                                "  Will resume: {} ({} objects, {:.1}s elapsed)",
                                rt, count, elapsed
                            );
                        }
                    }
                    let arm: HashMap<_, _> = all_state
                        .into_iter()
                        .filter(|(k, _)| k.starts_with("arm:"))
                        .collect();
                    if !arm.is_empty() {
                        info!(
                            "Resuming ARM enumeration with {} completed unit(s)",
                            arm.len()
                        );
                        for (rt, (status, count, elapsed)) in &arm {
                            debug!(
                                "  Saved: {} (status={}, {} objects, {:.1}s elapsed)",
                                rt, status, count, elapsed
                            );
                        }
                    }
                    (graph, arm)
                }
                Err(e) => {
                    debug!("Could not read state (fresh DB?): {}", e);
                    (HashMap::new(), HashMap::new())
                }
            };
            gs
        }
    };

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
        match graph_collector.msgraph_credential.get_token().await {
            Ok(_) => {
                debug!("MS Graph token retrieved successfully");

                // Create a task for Graph enumeration
                let graph_collector_clone = Arc::clone(&graph_collector);
                let graph_task = tokio::spawn(async move {
                    debug!("Starting Graph API enumeration task");
                    if let Err(e) = graph::enumerate_graph(graph_collector_clone, graph_state).await
                    {
                        error!("Error collecting Graph data: {}", e);
                    }
                    debug!("Graph API enumeration task completed");
                });

                tasks.push(graph_task);
            }
            Err(e) => {
                warn!(
                    "MS Graph credential failed \u{2014} skipping Graph enumeration: {}",
                    e
                );
            }
        }
    }

    // Configure and prepare ARM API enumeration task
    if arm_collector.mode == EnumerationMode::Arm || arm_collector.mode == EnumerationMode::Both {
        debug!("ARM enumeration mode is enabled");

        // Pre-fetch token to validate credentials
        match arm_collector.arm_credential.get_token().await {
            Ok(_) => {
                debug!("ARM token retrieved successfully");

                // Create a task for ARM enumeration
                let arm_collector_clone = Arc::clone(&arm_collector);
                let arm_task = tokio::spawn(async move {
                    debug!("Starting ARM API enumeration task");
                    if let Err(e) = arm::enumerate_arm(arm_collector_clone, arm_state).await {
                        error!("Error collecting ARM data: {}", e);
                    }
                    debug!("ARM API enumeration task completed");
                });

                tasks.push(arm_task);
            }
            Err(e) => {
                warn!(
                    "ARM credential failed \u{2014} skipping ARM enumeration: {}",
                    e
                );
            }
        }
    }

    // Wait for all tasks to complete
    if !tasks.is_empty() {
        debug!("Waiting for {} enumeration tasks to complete", tasks.len());
        futures::future::join_all(tasks).await;
        info!("All enumeration tasks completed");
    }

    // Wait for the DB writer to finish
    info!("Sending shutdown message to DB writer");
    db_tx
        .send(DBWriteMessage::Shutdown)
        .await
        .map_err(|e| CirroError::DatabaseError(e.to_string()))?;

    db_writer_task
        .await
        .map_err(|e| CirroError::DatabaseError(e.to_string()))?;

    info!("Done");
    Ok(())
}

/// Collects data using Access Token credentials
pub async fn collect_with_access_token(
    token: String,
    cloud: AzureCloud,
    output_path: PathBuf,
    option_enum_flags: OptionEnumFlags,
    new_state: bool,
    subscription_ids: Option<Vec<String>>,
    subscription_concurrency: usize,
) -> Result<(), CirroError> {
    // We don't really need to create all of these credentials for Access Token mode,
    // but we do it to keep the interface consistent with other authentication modes.
    // We can probably optimize this later.
    let mut arm_credential = Box::new(
        crate::collect::azure::credentials::access_token::AccessTokenCredential {
            token: tokio::sync::RwLock::new(
                crate::collect::azure::credentials::common::Token::default(),
            ),
        },
    );
    let mut msgraph_credential = Box::new(
        crate::collect::azure::credentials::access_token::AccessTokenCredential {
            token: tokio::sync::RwLock::new(
                crate::collect::azure::credentials::common::Token::default(),
            ),
        },
    );
    let vault_credential = Box::new(
        crate::collect::azure::credentials::access_token::AccessTokenCredential {
            token: tokio::sync::RwLock::new(
                crate::collect::azure::credentials::common::Token::default(),
            ),
        },
    );
    let storage_credential = Box::new(
        crate::collect::azure::credentials::access_token::AccessTokenCredential {
            token: tokio::sync::RwLock::new(
                crate::collect::azure::credentials::common::Token::default(),
            ),
        },
    );

    // Get the token audience and compute the cloud endpoints
    // This is necessary to ensure the correct resource URLs are used for the token.
    let cloud_endpoints: CloudEndpoints = CloudEndpoints::new(cloud);
    let mut token = crate::collect::azure::credentials::common::Token {
        access_token: Arc::new(token),
        expires_on: None,
        refresh_token: None,
    };
    token
        .set_expires_on_from_token()
        .map_err(|e| CirroError::AuthError(format!("Failed to set expires_on: {}", e)))?;
    let credential = Box::new(
        crate::collect::azure::credentials::access_token::AccessTokenCredential {
            token: tokio::sync::RwLock::new(token.clone()),
        },
    );

    // Get audience claim
    let audience = credential.get_token().await?.get_claims().unwrap().aud;
    let mode: EnumerationMode;

    info!("Using token with audience: {}", audience);
    match audience.as_str() {
        // Audience can sometimes be the App ID URI or the resource URL
        // Audience might have a trailing slash or not, so we handle both cases
        _ if audience.trim_end_matches("/") == cloud_endpoints.arm_url.trim_end_matches('/')
            || audience == "00000002-0000-0000-c000-000000000000"
            || audience.trim_end_matches("/") == "https://management.core.windows.net" =>
        {
            mode = EnumerationMode::Arm;
            arm_credential = credential;
        }
        _ if audience.trim_end_matches("/")
            == cloud_endpoints.msgraph_url.trim_end_matches('/')
            || audience == "00000003-0000-0000-c000-000000000000" =>
        {
            mode = EnumerationMode::Graph;
            msgraph_credential = credential;
        }
        _ => {
            return Err(CirroError::AuthError(format!("Invalid token audience",)));
        }
    }

    info!("Starting with Access Token credentials");

    // Initialize the collector context with Access Token credentials
    let collector = Collector {
        context: CollectorContext::<Box<dyn AuthCredential + Send + Sync + 'static>> {
            mode,
            option_enum_flags,
            subscription_concurrency,
            cloud,
            cloud_endpoints: CloudEndpoints::new(cloud),
            msgraph_credential,
            arm_credential,
            vault_credential,
            storage_credential,
            output_path,
            tenant_id: None,
            subscription_id: None,
            client_id: None,
            client_secret: None,
            client_cert_path: None,
            subscription_filter: subscription_ids.map(|ids| ids.into_iter().collect()),
            db_writer: None,
        },
    };

    if let Err(e) = run_collector(collector, new_state).await {
        error!("Error collecting data: {}", e);
        return Err(e);
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
    option_enum_flags: OptionEnumFlags,
    new_state: bool,
    subscription_ids: Option<Vec<String>>,
    subscription_concurrency: usize,
) -> Result<(), CirroError> {
    info!("Starting with Azure CLI credentials");

    let cloud_endpoints = CloudEndpoints::new(cloud);

    // Initialize the Azure CLI credentials
    let arm_credential: Box<dyn AuthCredential + Send + Sync> = Box::new(AzureCliCredential {
        tenant_id: tenant_id.clone(),
        resource: &cloud_endpoints.arm_url,
        token: tokio::sync::RwLock::new(
            crate::collect::azure::credentials::common::Token::default(),
        ),
    });
    let msgraph_credential: Box<dyn AuthCredential + Send + Sync> = Box::new(AzureCliCredential {
        tenant_id: tenant_id.clone(),
        resource: &cloud_endpoints.msgraph_url,
        token: tokio::sync::RwLock::new(
            crate::collect::azure::credentials::common::Token::default(),
        ),
    });
    let vault_credential: Box<dyn AuthCredential + Send + Sync> = Box::new(AzureCliCredential {
        tenant_id: tenant_id.clone(),
        resource: &cloud_endpoints.vault_url,
        token: tokio::sync::RwLock::new(
            crate::collect::azure::credentials::common::Token::default(),
        ),
    });
    let storage_credential: Box<dyn AuthCredential + Send + Sync> = Box::new(AzureCliCredential {
        tenant_id: tenant_id.clone(),
        resource: &cloud_endpoints.storage_url,
        token: tokio::sync::RwLock::new(
            crate::collect::azure::credentials::common::Token::default(),
        ),
    });

    // Initialize the collector context with Azure CLI credentials
    let collector = Collector {
        context: CollectorContext::<Box<dyn AuthCredential + Send + Sync + 'static>> {
            mode,
            option_enum_flags,
            subscription_concurrency,
            cloud,
            cloud_endpoints: CloudEndpoints::new(cloud),
            msgraph_credential,
            arm_credential,
            vault_credential,
            storage_credential,
            output_path: output_path,
            tenant_id: tenant_id.clone(),
            subscription_id,
            client_id: None,
            client_secret: None,
            client_cert_path: None,
            subscription_filter: subscription_ids.map(|ids| ids.into_iter().collect()),
            db_writer: None,
        },
    };

    if let Err(e) = run_collector(collector, new_state).await {
        error!("Error collecting data: {}", e);
        return Err(e);
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
    option_enum_flags: OptionEnumFlags,
    new_state: bool,
    subscription_ids: Option<Vec<String>>,
    subscription_concurrency: usize,
) -> Result<(), CirroError> {
    info!("Starting with Client Secret credentials");

    let cloud_endpoints = CloudEndpoints::new(cloud);

    // Initialize the Client Secret credentials
    let arm_credential = Box::new(
        crate::collect::azure::credentials::clientsecret::ClientSecretCredential {
            token_endpoint: &cloud_endpoints.token_endpoint,
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
            tenant_id: tenant_id.clone(),
            resource: &cloud_endpoints.arm_url,
            token: tokio::sync::RwLock::new(
                crate::collect::azure::credentials::common::Token::default(),
            ),
        },
    );
    let msgraph_credential = Box::new(
        crate::collect::azure::credentials::clientsecret::ClientSecretCredential {
            token_endpoint: &cloud_endpoints.token_endpoint,
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
            tenant_id: tenant_id.clone(),
            resource: cloud_endpoints.msgraph_url,
            token: tokio::sync::RwLock::new(
                crate::collect::azure::credentials::common::Token::default(),
            ),
        },
    );
    let vault_credential = Box::new(
        crate::collect::azure::credentials::clientsecret::ClientSecretCredential {
            token_endpoint: &cloud_endpoints.token_endpoint,
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
            tenant_id: tenant_id.clone(),
            resource: cloud_endpoints.vault_url,
            token: tokio::sync::RwLock::new(
                crate::collect::azure::credentials::common::Token::default(),
            ),
        },
    );
    let storage_credential = Box::new(
        crate::collect::azure::credentials::clientsecret::ClientSecretCredential {
            token_endpoint: &cloud_endpoints.token_endpoint,
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
            tenant_id: tenant_id.clone(),
            resource: cloud_endpoints.storage_url,
            token: tokio::sync::RwLock::new(
                crate::collect::azure::credentials::common::Token::default(),
            ),
        },
    );

    // Initialize the collector context with Client Secret credentials
    let collector = Collector {
        context: CollectorContext::<Box<dyn AuthCredential + Send + Sync + 'static>> {
            mode,
            option_enum_flags,
            subscription_concurrency,
            cloud,
            cloud_endpoints: CloudEndpoints::new(cloud),
            msgraph_credential,
            arm_credential,
            vault_credential,
            storage_credential,
            output_path,
            tenant_id: Some(tenant_id.clone()),
            subscription_id: None,
            client_id: None,
            client_secret: None,
            client_cert_path: None,
            subscription_filter: subscription_ids.map(|ids| ids.into_iter().collect()),
            db_writer: None,
        },
    };

    if let Err(e) = run_collector(collector, new_state).await {
        error!("Error collecting data: {}", e);
        return Err(e);
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
    option_enum_flags: OptionEnumFlags,
    new_state: bool,
    subscription_ids: Option<Vec<String>>,
    subscription_concurrency: usize,
) -> Result<(), CirroError> {
    info!("Starting with Client Certificate credentials");

    let cloud_endpoints = CloudEndpoints::new(cloud);

    // Initialize the Client Certificate credentials
    let arm_credential = Box::new(
        crate::collect::azure::credentials::clientcert::ClientCertificateCredential {
            token_endpoint: &cloud_endpoints.token_endpoint,
            client_id: client_id.clone(),
            certificate_path: client_certificate.clone(),
            tenant_id: tenant_id.clone(),
            resource: &cloud_endpoints.arm_url,
            token: tokio::sync::RwLock::new(
                crate::collect::azure::credentials::common::Token::default(),
            ),
        },
    );
    let msgraph_credential = Box::new(
        crate::collect::azure::credentials::clientcert::ClientCertificateCredential {
            token_endpoint: &cloud_endpoints.token_endpoint,
            client_id: client_id.clone(),
            certificate_path: client_certificate.clone(),
            tenant_id: tenant_id.clone(),
            resource: cloud_endpoints.msgraph_url,
            token: tokio::sync::RwLock::new(
                crate::collect::azure::credentials::common::Token::default(),
            ),
        },
    );
    let vault_credential = Box::new(
        crate::collect::azure::credentials::clientcert::ClientCertificateCredential {
            token_endpoint: &cloud_endpoints.token_endpoint,
            client_id: client_id.clone(),
            certificate_path: client_certificate.clone(),
            tenant_id: tenant_id.clone(),
            resource: cloud_endpoints.vault_url,
            token: tokio::sync::RwLock::new(
                crate::collect::azure::credentials::common::Token::default(),
            ),
        },
    );
    let storage_credential = Box::new(
        crate::collect::azure::credentials::clientcert::ClientCertificateCredential {
            token_endpoint: &cloud_endpoints.token_endpoint,
            client_id: client_id.clone(),
            certificate_path: client_certificate.clone(),
            tenant_id: tenant_id.clone(),
            resource: cloud_endpoints.storage_url,
            token: tokio::sync::RwLock::new(
                crate::collect::azure::credentials::common::Token::default(),
            ),
        },
    );

    // Initialize the collector context with Client Certificate credentials
    let collector = Collector {
        context: CollectorContext::<Box<dyn AuthCredential + Send + Sync + 'static>> {
            mode,
            option_enum_flags,
            subscription_concurrency,
            cloud,
            cloud_endpoints: CloudEndpoints::new(cloud),
            msgraph_credential,
            arm_credential,
            vault_credential,
            storage_credential,
            output_path,
            tenant_id: Some(tenant_id.clone()),
            subscription_id: None,
            client_id: Some(client_id),
            client_secret: None,
            client_cert_path: Some(client_certificate),
            subscription_filter: subscription_ids.map(|ids| ids.into_iter().collect()),
            db_writer: None,
        },
    };
    // Run the collector

    if let Err(e) = run_collector(collector, new_state).await {
        error!("Error collecting data: {}", e);
        return Err(e);
    }

    Ok(())
}
