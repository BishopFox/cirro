use crate::collect::azure::cli::{AzureCloud, EnumerationMode, OptionEnumFlags};
use crate::collect::azure::db::{ArmResourceMessage, DBWriteMessage, DataMessage};
use crate::errors::CirroError;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct CollectorContext<C> {
    pub mode: EnumerationMode,
    pub option_enum_flags: OptionEnumFlags,
    pub subscription_concurrency: usize,
    pub cloud: AzureCloud,
    pub cloud_endpoints: CloudEndpoints,
    pub msgraph_credential: C,
    pub arm_credential: C,
    pub vault_credential: C,
    pub storage_credential: C,
    pub output_path: PathBuf,
    pub tenant_id: Option<String>,
    pub subscription_id: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub client_cert_path: Option<PathBuf>,
    pub subscription_filter: Option<HashSet<String>>,
    pub db_writer: Option<Arc<mpsc::Sender<DBWriteMessage>>>,
}

impl<C> CollectorContext<C> {
    /// Writes a value to the database using the provided table and id
    pub async fn write_value_to_db(
        &self,
        table: String,
        id: String,
        data: serde_json::Value,
    ) -> Result<(), CirroError> {
        if let Some(sender) = &self.db_writer {
            let message = DBWriteMessage::Data(DataMessage { table, id, data });
            sender
                .send(message)
                .await
                .map_err(|e| CirroError::DatabaseError(e.to_string()))?;
            Ok(())
        } else {
            Err(CirroError::DatabaseError("No DB writer available".into()))
        }
    }

    /// Writes a batch of values to one table in the database
    pub async fn write_values_batch_to_db(
        &self,
        table: String,
        rows: Vec<(String, serde_json::Value)>,
    ) -> Result<(), CirroError> {
        if let Some(sender) = &self.db_writer {
            let batch_rows = rows
                .into_iter()
                .map(|(id, data)| DataMessage {
                    table: table.clone(),
                    id,
                    data,
                })
                .collect();

            let message = DBWriteMessage::DataBatch(batch_rows);
            sender
                .send(message)
                .await
                .map_err(|e| CirroError::DatabaseError(e.to_string()))?;
            Ok(())
        } else {
            Err(CirroError::DatabaseError("No DB writer available".into()))
        }
    }

    /// Writes an ARM resource to the database
    /// This includes the resource type, subscription ID, resource group ID, and data
    pub async fn write_arm_resource_to_db(
        &self,
        id: String,
        sub_id: String,
        rg_id: String,
        resource_type: String,
        data: serde_json::Value,
    ) -> Result<(), CirroError> {
        if let Some(sender) = &self.db_writer {
            let message = DBWriteMessage::ArmResource(ArmResourceMessage {
                id,
                sub_id,
                rg_id,
                resource_type,
                data,
            });
            sender
                .send(message)
                .await
                .map_err(|e| CirroError::DatabaseError(e.to_string()))?;
            Ok(())
        } else {
            Err(CirroError::DatabaseError("No DB writer available".into()))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CloudEndpoints {
    pub token_endpoint: &'static str,
    pub msgraph_url: &'static str,
    pub arm_url: &'static str,
    pub vault_url: &'static str,
    pub storage_url: &'static str,
}

impl CloudEndpoints {
    /// Creates a new instance of `CloudEndpoints` with the provided URLs from Cloud enum
    pub fn new(cloud: AzureCloud) -> Self {
        match cloud {
            AzureCloud::Public => CloudEndpoints {
                token_endpoint: "https://login.microsoftonline.com",
                msgraph_url: "https://graph.microsoft.com",
                arm_url: "https://management.azure.com",
                vault_url: "https://vault.azure.net",
                storage_url: "https://storage.azure.com",
            },
            AzureCloud::China => CloudEndpoints {
                token_endpoint: "https://login.chinacloudapi.cn",
                msgraph_url: "https://microsoftgraph.chinacloudapi.cn",
                arm_url: "https://management.chinacloudapi.cn",
                vault_url: "https://vault.azure.cn",
                storage_url: "https://storage.azure.cn",
            },
            AzureCloud::Germany => CloudEndpoints {
                token_endpoint: "https://login.microsoftazure.de",
                msgraph_url: "https://microsoftgraph.microsoft.de",
                arm_url: "https://management.microsoftazure.de",
                vault_url: "https://vault.microsoftazure.de",
                storage_url: "https://storage.microsoftazure.de",
            },
            AzureCloud::USGov => CloudEndpoints {
                token_endpoint: "https://login.microsoftonline.us",
                msgraph_url: "https://graph.microsoft.us",
                arm_url: "https://management.usgovcloudapi.net",
                vault_url: "https://vault.azure.us",
                storage_url: "https://storage.azure.us",
            },
        }
    }
}
