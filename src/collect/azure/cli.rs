use crate::collect::azure;
use crate::collect::azure::cli_utils::{validate_tenant_id, validate_uuid};
use crate::collect::logger::setup_logger;
use crate::errors::CirroError;

use clap::{Args, Subcommand};
use std::{collections::HashSet, path::PathBuf};

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

    /// Clear saved pagination state and start a fresh enumeration
    #[arg(long = "new-state", action = clap::ArgAction::SetTrue)]
    pub new_state: bool,

    /// Filter ARM collection to specific subscription IDs (comma-separated or repeated)
    #[arg(long = "subscription-ids", value_name = "ID", value_delimiter = ',', num_args = 1.., value_parser = validate_uuid)]
    pub subscription_ids: Option<Vec<String>>,

    /// Limit subscription concurrency to avoid throttling (default: 5)
    #[arg(long, value_name = "N", default_value_t = 5)]
    pub subscription_concurrency: usize,
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

    /// Clear saved pagination state and start a fresh enumeration
    #[arg(long = "new-state", action = clap::ArgAction::SetTrue)]
    pub new_state: bool,

    /// Filter ARM collection to specific subscription IDs (comma-separated or repeated)
    #[arg(long = "subscription-ids", value_name = "ID", value_delimiter = ',', num_args = 1.., value_parser = validate_uuid)]
    pub subscription_ids: Option<Vec<String>>,

    /// Limit subscription concurrency to avoid throttling (default: 5)
    #[arg(long, value_name = "N", default_value_t = 5)]
    pub subscription_concurrency: usize,
}

#[derive(Debug, Clone, Args)]
pub struct OptionEnumFlags {
    /// Skip all default graph enumerations; only run explicitly enabled options (e.g. --pim, --caps)
    #[arg(long = "no-default", action = clap::ArgAction::SetTrue)]
    pub no_default: bool,

    /// Enumerate only graph organization details
    #[arg(long = "organization", action = clap::ArgAction::SetTrue)]
    pub organization: bool,

    /// Enumerate only graph authorization policy details
    #[arg(long = "authorization-policy", action = clap::ArgAction::SetTrue)]
    pub authorization_policy: bool,

    /// Enumerate only graph directory users
    #[arg(long = "users", action = clap::ArgAction::SetTrue)]
    pub users: bool,

    /// Enumerate only graph directory groups
    #[arg(long = "groups", action = clap::ArgAction::SetTrue)]
    pub groups: bool,

    /// Enumerate only graph application registrations
    #[arg(long = "applications", action = clap::ArgAction::SetTrue)]
    pub applications: bool,

    /// Enumerate only graph service principals
    #[arg(long = "service-principals", action = clap::ArgAction::SetTrue)]
    pub service_principals: bool,

    /// Enumerate only graph registered devices
    #[arg(long = "devices", action = clap::ArgAction::SetTrue)]
    pub devices: bool,

    /// Enumerate only graph directory role definitions
    #[arg(long = "directory-roles", action = clap::ArgAction::SetTrue)]
    pub directory_roles: bool,

    /// Enumerate only graph administrative units
    #[arg(long = "administrative-units", action = clap::ArgAction::SetTrue)]
    pub administrative_units: bool,

    /// Enumerate only oauth2 delegated permission grants
    #[arg(long = "oauth2-grants", action = clap::ArgAction::SetTrue)]
    pub oauth2_grants: bool,

    /// Gather eligible Graph role assignments for users (requires permissions)
    #[arg(long = "graph-pim", action = clap::ArgAction::SetTrue)]
    pub graph_pim: bool,

    /// Gather eligible ARM role assignments for current user (user only)
    #[arg(long = "arm-pim", action = clap::ArgAction::SetTrue)]
    pub arm_pim: bool,

    /// Gather conditional access policies (requires permissions)
    #[arg(long = "caps", action = clap::ArgAction::SetTrue)]
    pub caps: bool,
}

impl OptionEnumFlags {
    /// Returns a filter set of graph object names when users explicitly request them
    pub fn graph_object_filters(&self) -> Option<HashSet<String>> {
        let mut selected: HashSet<String> = HashSet::new();

        if self.organization {
            selected.insert("organization".to_string());
        }
        if self.authorization_policy {
            selected.insert("authorizationPolicy".to_string());
        }
        if self.users {
            selected.insert("users".to_string());
        }
        if self.groups {
            selected.insert("groups".to_string());
        }
        if self.applications {
            selected.insert("applications".to_string());
        }
        if self.service_principals {
            selected.insert("servicePrincipals".to_string());
        }
        if self.devices {
            selected.insert("devices".to_string());
        }
        if self.directory_roles {
            selected.insert("directoryRoles".to_string());
        }
        if self.administrative_units {
            selected.insert("administrativeUnits".to_string());
        }
        if self.oauth2_grants {
            selected.insert("oauth2PermissionGrants".to_string());
        }

        if selected.is_empty() && !self.no_default {
            None
        } else {
            Some(selected)
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum AzureCommands {
    /// Authenticate using an access token
    AccessToken {
        /// Access token
        #[arg(short, long)]
        token: String,

        #[clap(flatten)]
        common: AccessTokenAuthArgs,

        #[clap(flatten)]
        option_enum_flags: OptionEnumFlags,
    },
    /// Authenticate using Azure CLI
    Azcli {
        /// Tenant ID
        #[arg(short, long, value_parser = validate_tenant_id)]
        tenant_id: Option<String>,

        /// Subscription ID for az login authentication context
        #[arg(short = 's', long = "login-subscription-id", value_parser = validate_uuid)]
        login_subscription_id: Option<String>,

        #[clap(flatten)]
        common: CommonAuthArgs,

        #[clap(flatten)]
        option_enum_flags: OptionEnumFlags,
    },
    /// Authenticate using a client secret
    ClientSecret {
        /// Client ID
        #[arg(short, long, value_parser = validate_uuid)]
        client_id: String,

        /// Client secret
        #[arg(short = 'p', long)]
        client_secret: String,

        /// Tenant ID
        #[arg(short, long, value_parser = validate_tenant_id)]
        tenant_id: String,

        #[clap(flatten)]
        common: CommonAuthArgs,

        #[clap(flatten)]
        option_enum_flags: OptionEnumFlags,
    },
    /// Authenticate using a client certificate
    ClientCert {
        /// Client ID
        #[arg(short, long, value_parser = validate_uuid)]
        client_id: String,

        /// Path to the client certificate file (PEM format)
        #[arg(short = 'p', long = "certificate", value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
        client_certificate: PathBuf,

        /// Tenant ID
        #[arg(short, long, value_parser = validate_tenant_id)]
        tenant_id: String,

        #[clap(flatten)]
        common: CommonAuthArgs,

        #[clap(flatten)]
        option_enum_flags: OptionEnumFlags,
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

        #[clap(flatten)]
        option_enum_flags: OptionEnumFlags,
    },
}

pub async fn handle_azure_command(command: AzureCommands) -> Result<(), CirroError> {
    match command {
        AzureCommands::AccessToken {
            token,
            common,
            option_enum_flags,
        } => {
            if let Err(e) = setup_logger(common.debug) {
                return Err(CirroError::Unknown(e.to_string()));
            }
            azure::collect::collect_with_access_token(
                token,
                common.cloud,
                common.output_path,
                option_enum_flags,
                common.new_state,
                common.subscription_ids,
                common.subscription_concurrency,
            )
            .await
        }
        AzureCommands::Azcli {
            tenant_id,
            login_subscription_id,
            common,
            option_enum_flags,
        } => {
            if let Err(e) = setup_logger(common.debug) {
                return Err(CirroError::Unknown(e.to_string()));
            }
            azure::collect::collect_with_azure_cli(
                tenant_id,
                login_subscription_id,
                common.mode,
                common.cloud,
                common.output_path,
                option_enum_flags,
                common.new_state,
                common.subscription_ids,
                common.subscription_concurrency,
            )
            .await
        }
        AzureCommands::ClientSecret {
            client_id,
            client_secret,
            tenant_id,
            common,
            option_enum_flags,
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
                option_enum_flags,
                common.new_state,
                common.subscription_ids,
                common.subscription_concurrency,
            )
            .await
        }
        AzureCommands::ClientCert {
            client_id,
            client_certificate,
            tenant_id,
            common,
            option_enum_flags,
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
                option_enum_flags,
                common.new_state,
                common.subscription_ids,
                common.subscription_concurrency,
            )
            .await
        }
        AzureCommands::UserPass {
            upn: _,
            password: _,
            common,
            option_enum_flags: _,
        } => {
            if let Err(e) = setup_logger(common.debug) {
                return Err(CirroError::Unknown(e.to_string()));
            }
            Err(CirroError::Unknown(
                "Username/password authentication not yet implemented".to_string(),
            ))
        }
    }
}
