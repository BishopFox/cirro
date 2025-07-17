pub mod collect;
pub mod collectors;
pub mod context;
pub mod credentials;
pub mod db;
pub mod errors;
pub mod logger;
pub mod styles;

use crate::errors::CirroError;
use crate::logger::setup_logger;

use clap::{
    Args, ColorChoice, Parser, Subcommand, crate_authors, crate_description, crate_version,
};
use colored::Colorize;
use std::path::PathBuf;
use tokio;

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

#[derive(Debug, Parser)]
#[clap(
    about = crate_description!(),
    version = crate_version!(),
    bin_name = "cirro",
    author = crate_authors!(),
    styles = styles::get_styles(),
    color = ColorChoice::Always,
    arg_required_else_help = true,
)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Collect data from Azure and Microsoft Graph APIs
    Collect {
        #[command(subcommand)]
        auth_mode: AuthenticationMode,
    },
    /// Enrich data in the database
    Enrich {
        #[command(subcommand)]
        auth_mode: AuthenticationMode,

        #[clap(flatten)]
        flags: EnrichmentFlags,
    },
}

#[derive(Debug, Clone, Args)]
struct CommonAuthArgs {
    /// Output database file path
    #[arg(short, long, value_name = "FILE", global=true, default_value = "cirro_output.db", value_hint = clap::ValueHint::FilePath)]
    output_path: PathBuf,

    /// Enumeration mode
    #[arg(
        short,
        long,
        value_enum,
        ignore_case = true,
        default_value = "both",
        value_name = "MODE"
    )]
    mode: EnumerationMode,

    /// Cloud to enumerate
    #[arg(long, value_enum, ignore_case = true, default_value = "public")]
    cloud: AzureCloud,

    /// Debug output
    #[arg(long = "debug", action = clap::ArgAction::SetTrue)]
    debug: bool,
}

#[derive(Debug, Clone, Args)]
struct AccessTokenAuthArgs {
    /// Output database file path
    #[arg(short, long, value_name = "FILE", global=true, default_value = "cirro_output.db", value_hint = clap::ValueHint::FilePath)]
    output_path: PathBuf,

    /// Cloud to enumerate
    #[arg(long, value_enum, ignore_case = true, default_value = "public")]
    cloud: AzureCloud,

    /// Debug output
    #[arg(long = "debug", action = clap::ArgAction::SetTrue)]
    debug: bool,
}

#[derive(Debug, Subcommand)]
enum AuthenticationMode {
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

#[derive(Debug, Clone, Args)]
pub struct EnrichmentFlags {
    /// Gather storage account keys
    #[arg(long, action = clap::ArgAction::SetTrue)]
    storage_keys: bool,

    /// Gather storage account containers and blobs with authentication
    #[arg(long, action = clap::ArgAction::SetTrue)]
    storage_blobs: bool,

    /// Gather storage account containers and blobs without authentication
    #[arg(long, action = clap::ArgAction::SetTrue)]
    storage_blobs_anon: bool,
}

#[tokio::main]
async fn main() -> Result<(), CirroError> {
    let cli = Cli::parse();

    let logo = r#"
   ___      _                            
  / __|    (_)      _ _     _ _    ___   
 | (__     | |     | '_|   | '_|  / _ \  
  \___|   _|_|_   _|_|_   _|_|_   \___/  
_|"""""|_|"""""|_|"""""|_|"""""|_|"""""| 
"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-' 
        "#;

    println!("{}", logo.blue().bold());

    match cli.command {
        Some(Commands::Collect { auth_mode }) => match auth_mode {
            AuthenticationMode::AccessToken { token, common } => {
                // Get verbose flag from CLI
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                if let Err(e) = collect::collect_with_access_token(
                    token,
                    common.cloud,
                    common.output_path,
                    false,
                    None,
                )
                .await
                {
                    return Err(e);
                }
            }

            AuthenticationMode::Azcli {
                tenant_id,
                subscription_id,
                common,
            } => {
                // Get verbose flag from CLI
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                if let Err(e) = collect::collect_with_azure_cli(
                    tenant_id,
                    subscription_id,
                    common.mode,
                    common.cloud,
                    common.output_path,
                    false,
                    None,
                )
                .await
                {
                    return Err(e);
                }
            }
            AuthenticationMode::ClientSecret {
                client_id,
                client_secret,
                tenant_id,
                common,
            } => {
                // Get verbose flag from CLI
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                if let Err(e) = collect::collect_with_client_secret(
                    tenant_id,
                    client_id,
                    client_secret,
                    common.mode,
                    common.cloud,
                    common.output_path,
                    false,
                    None,
                )
                .await
                {
                    return Err(e);
                }
            }
            AuthenticationMode::ClientCert {
                client_id,
                client_certificate,
                tenant_id,
                common,
            } => {
                // Get verbose flag from CLI
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                if let Err(e) = collect::collect_with_client_cert(
                    tenant_id,
                    client_id,
                    client_certificate,
                    common.mode,
                    common.cloud,
                    common.output_path,
                    false,
                    None,
                )
                .await
                {
                    return Err(e);
                }
            }
            _ => {
                return Err(CirroError::Unknown(
                    "Unsupported authentication mode".to_string(),
                ));
            }
        },
        Some(Commands::Enrich { auth_mode, flags }) => match auth_mode {
            AuthenticationMode::AccessToken { token, common } => {
                // Get verbose flag from CLI
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                if let Err(e) = collect::collect_with_access_token(
                    token,
                    common.cloud,
                    common.output_path,
                    true,
                    Some(flags),
                )
                .await
                {
                    return Err(e);
                }
            }

            AuthenticationMode::Azcli {
                tenant_id,
                subscription_id,
                common,
            } => {
                // Get verbose flag from CLI
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                if let Err(e) = collect::collect_with_azure_cli(
                    tenant_id,
                    subscription_id,
                    common.mode,
                    common.cloud,
                    common.output_path,
                    true,
                    Some(flags),
                )
                .await
                {
                    return Err(e);
                }
            }
            AuthenticationMode::ClientSecret {
                client_id,
                client_secret,
                tenant_id,
                common,
            } => {
                // Get verbose flag from CLI
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                if let Err(e) = collect::collect_with_client_secret(
                    tenant_id,
                    client_id,
                    client_secret,
                    common.mode,
                    common.cloud,
                    common.output_path,
                    true,
                    Some(flags),
                )
                .await
                {
                    return Err(e);
                }
            }
            AuthenticationMode::ClientCert {
                client_id,
                client_certificate,
                tenant_id,
                common,
            } => {
                // Get verbose flag from CLI
                if let Err(e) = setup_logger(common.debug) {
                    return Err(CirroError::Unknown(e.to_string()));
                }
                if let Err(e) = collect::collect_with_client_cert(
                    tenant_id,
                    client_id,
                    client_certificate,
                    common.mode,
                    common.cloud,
                    common.output_path,
                    true,
                    Some(flags),
                )
                .await
                {
                    return Err(e);
                }
            }
            _ => {
                return Err(CirroError::Unknown(
                    "Unsupported authentication mode".to_string(),
                ));
            }
        },
        None => {}
    }

    Ok(())
}
