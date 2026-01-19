use crate::azure;
use crate::errors::CirroError;
use crate::logger::setup_logger;

use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(clap::ValueEnum, Copy, Clone, Debug)]
pub enum AzureCloud {
    Public,
    China,
    Germany,
    USGov,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum EnumerationMode {
    Both,
    Graph,
    Arm,
}

#[derive(Debug, Subcommand)]
pub enum AzureCommands {
    /// Collect data from Azure and Microsoft Graph APIs
    Collect {
        #[command(subcommand)]
        auth_mode: AuthenticationMode,
    },
}

#[derive(Debug, Clone, Args)]
pub struct CommonAuthArgs {
    /// Output database file path
    #[arg(short, long, value_name = "FILE", global=true, default_value = "cirro_output.db", value_hint = clap::ValueHint::FilePath)]
    pub output_path: PathBuf,

    /// Enumeration mode
    #[arg(
        short,
        long,
        value_enum,
        ignore_case = true,
        default_value = "both",
        value_name = "MODE"
    )]
    pub mode: EnumerationMode,

    /// Cloud to enumerate
    #[arg(long, value_enum, ignore_case = true, default_value = "public")]
    pub cloud: AzureCloud,

    /// Debug output
    #[arg(long = "debug", action = clap::ArgAction::SetTrue)]
    pub debug: bool,
}

#[derive(Debug, Clone, Args)]
pub struct AccessTokenAuthArgs {
    /// Output database file path
    #[arg(short, long, value_name = "FILE", global=true, default_value = "cirro_output.db", value_hint = clap::ValueHint::FilePath)]
    pub output_path: PathBuf,

    /// Cloud to enumerate
    #[arg(long, value_enum, ignore_case = true, default_value = "public")]
    pub cloud: AzureCloud,

    /// Debug output
    #[arg(long = "debug", action = clap::ArgAction::SetTrue)]
    pub debug: bool,
}

#[derive(Debug, Subcommand)]
pub enum AuthenticationMode {
    /// Authenticate using an access token
    AccessToken {
        /// Access token
        #[arg(short, long)]
        token: String,
        #[clap(flatten)]
        common: AccessTokenAuthArgs,
    },
    /// Authenticate using Azure CLI
    Azcli {
        /// Tenant ID
        #[arg(short, long)]
        tenant_id: Option<String>,

        /// Subscription ID
        #[arg(short, long)]
        subscription_id: Option<String>,
        #[clap(flatten)]
        common: CommonAuthArgs,
    },
    /// Authenticate using a client secret
    ClientSecret {
        /// Client ID
        #[arg(short, long)]
        client_id: String,

        /// Client secret
        #[arg(short = 'p', long)]
        client_secret: String,

        /// Tenant ID
        #[arg(short, long)]
        tenant_id: String,

        #[clap(flatten)]
        common: CommonAuthArgs,
    },
    /// Authenticate using a client certificate
    ClientCert {
        /// Client ID
        #[arg(short, long)]
        client_id: String,

        /// Path to the client certificate file (PEM format)
        #[arg(short = 'p', long = "certificate", value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
        client_certificate: PathBuf,

        /// Tenant ID
        #[arg(short, long)]
        tenant_id: String,

        #[clap(flatten)]
        common: CommonAuthArgs,
    },
    /// Authenticate using a username and password
    UserPass {
        /// The username of the account (UPN format)
        #[arg(short, long)]
        upn: String,

        /// The password of the account
        #[arg(short, long)]
        password: String,

        #[clap(flatten)]
        common: CommonAuthArgs,
    },
}

pub async fn handle_azure_command(command: AzureCommands) -> Result<(), CirroError> {
    match command {
        AzureCommands::Collect { auth_mode } => match auth_mode {
            AuthenticationMode::AccessToken { token, common } => {
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                azure::collect::collect_with_access_token(token, common.cloud, common.output_path)
                    .await
            }

            AuthenticationMode::Azcli {
                tenant_id,
                subscription_id,
                common,
            } => {
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                azure::collect::collect_with_azure_cli(
                    tenant_id,
                    subscription_id,
                    common.mode,
                    common.cloud,
                    common.output_path,
                )
                .await
            }

            AuthenticationMode::ClientSecret {
                client_id,
                client_secret,
                tenant_id,
                common,
            } => {
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                azure::collect::collect_with_client_secret(
                    tenant_id,
                    client_id,
                    client_secret,
                    common.mode,
                    common.cloud,
                    common.output_path,
                )
                .await
            }

            AuthenticationMode::ClientCert {
                client_id,
                client_certificate,
                tenant_id,
                common,
            } => {
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                azure::collect::collect_with_client_cert(
                    tenant_id,
                    client_id,
                    client_certificate,
                    common.mode,
                    common.cloud,
                    common.output_path,
                )
                .await
            }

            AuthenticationMode::UserPass {
                upn: _,
                password: _,
                common,
            } => {
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                // TODO: Implement user/pass authentication
                Err(CirroError::Unknown(
                    "Username/password authentication not yet implemented".to_string(),
                ))
            }
        },
    }
}
