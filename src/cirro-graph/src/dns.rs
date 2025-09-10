use crate::errors::CirroGraphError;

use csv::Writer;
use domain_check_lib::{CheckConfig, DomainChecker};
use hickory_resolver::TokioResolver;
use log::{debug, info, warn};
use lru::LruCache;
use neo4rs::*;
use psl;
use serde::Serialize;
use std::{fs::File, num::NonZeroUsize, path::PathBuf, sync::Mutex, time::Duration};
use tabled::{
    Table, Tabled,
    settings::{Panel, Style},
};

/// Represents the state of a domain during DNS checks
#[derive(Debug, Clone, Serialize, Tabled)]
pub struct DomainState {
    pub domain: String,
    pub is_nxdomain: bool,
    pub is_available: bool,
    pub root_domain: String,
    pub root_is_nxdomain: bool,
    pub root_is_available: bool,
}

/// Represents the state of a FederatedIdentityCredential during DNS checks
#[derive(Debug, Clone, Serialize, Tabled)]
pub struct FederatedIdentityCredentialState {
    pub app_id: String,
    pub display_name: String,
    pub publisher_domain: String,
    pub issuer: String,
    pub is_nxdomain: bool,
    pub is_available: bool,
}

/// Represents the state of a RedirectUri during DNS checks
#[derive(Debug, Clone, Serialize, Tabled)]
pub struct RedirectUriState {
    pub app_id: String,
    pub display_name: String,
    pub publisher_domain: String,
    pub sign_in_audience: String,
    pub redirect_uri: String,
    pub is_nxdomain: bool,
    pub is_available: bool,
}

/// Represents the state of a loginUrl during DNS checks
#[derive(Debug, Clone, Serialize, Tabled)]
pub struct LoginUrlState {
    pub app_id: String,
    pub display_name: String,
    pub login_url: String,
    pub is_nxdomain: bool,
    pub is_available: bool,
}

/// Represents the state of a ReplyUrl during DNS checks
#[derive(Debug, Clone, Serialize, Tabled)]
pub struct ReplyUrlState {
    pub app_id: String,
    pub app_org_id: String,
    pub display_name: String,
    pub reply_url: String,
    pub is_nxdomain: bool,
    pub is_available: bool,
}

pub struct DnsChecker {
    output_file: PathBuf,
    server: String,
    user: String,
    db_name: String,
    graph: Graph,
    whois_client: DomainChecker,
    nxdomain_cache: Mutex<LruCache<String, bool>>,
    availability_cache: Mutex<LruCache<String, bool>>,
}

/// Custom Debug trait for DnsChecker
impl std::fmt::Debug for DnsChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DnsChecker")
            .field("output_file", &self.output_file)
            .field("server", &self.server)
            .field("user", &self.user)
            .field("db_name", &self.db_name)
            .field("nxdomain_cache_size", &"1024")
            .field("availability_cache_size", &"1024")
            .finish()
    }
}

impl DnsChecker {
    pub async fn new(
        output_file: PathBuf,
        server: String,
        user: String,
        password: String,
        db_name: String,
    ) -> Self {
        let config = ConfigBuilder::default()
            .uri(&server)
            .user(&user)
            .password(&password)
            .db(db_name.clone())
            .build()
            .map_err(|e| CirroGraphError::DatabaseError(e.to_string()))
            .unwrap();

        info!("Connecting to database at {} with user {}", server, user);
        let graph = Graph::connect(config)
            .await
            .map_err(|e| CirroGraphError::DatabaseError(e.to_string()))
            .unwrap();

        // Test connection to the database
        let mut result = graph.execute(query("RETURN 1")).await.unwrap();
        let row = result.next().await.unwrap().unwrap();
        let value: i64 = row.get("1").unwrap();
        assert_eq!(1, value);
        info!("Successfully connected to the database");

        let config = CheckConfig::default()
            .with_concurrency(50) // Max 50 concurrent checks
            .with_timeout(Duration::from_secs(10)) // 10 second timeout
            .with_whois_fallback(false) // Enable WHOIS fallback
            .with_bootstrap(true) // Use IANA bootstrap
            .with_detailed_info(false); // Extract full domain info

        return DnsChecker {
            output_file,
            server,
            user,
            db_name,
            graph,
            whois_client: DomainChecker::with_config(config),
            nxdomain_cache: Mutex::new(LruCache::new(NonZeroUsize::new(4096).unwrap())),
            availability_cache: Mutex::new(LruCache::new(NonZeroUsize::new(4096).unwrap())),
        };
    }

    /// Helper function to save results to a CSV file
    fn save_to_csv<T: Serialize>(
        &self,
        results: &[T],
        filename: &str,
    ) -> Result<(), CirroGraphError> {
        let csv_path = self
            .output_file
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .join(filename);
        info!("Saving results to CSV file: {:?}", csv_path);

        let file = File::create(&csv_path)?;
        let mut writer = Writer::from_writer(file);

        for result in results {
            writer.serialize(result).map_err(|e| {
                CirroGraphError::DatabaseError(format!("Failed to write CSV record: {}", e))
            })?;
        }

        writer.flush().map_err(|e| {
            CirroGraphError::DatabaseError(format!("Failed to flush CSV writer: {}", e))
        })?;

        info!(
            "Successfully saved {} records to {:?}",
            results.len(),
            csv_path
        );
        Ok(())
    }

    /// Runs DNS checks
    pub async fn run(&mut self) -> Result<(), CirroGraphError> {
        info!("Running DNS checks...");

        // Print out the results as a table with a VerifiedDomains header
        let dns_results = self.check_verified_domains().await?;
        println!(
            "{}\n",
            Table::new(&dns_results)
                .with(Style::psql())
                .with(Panel::header("Verified Domains"))
        );
        self.save_to_csv(&dns_results, "verified_domains.csv")?;

        // Print out the results as a table with a FederatedIdentityCredentials header
        let fic_results = self.check_federated_identity_credentials().await?;
        println!(
            "{}\n",
            Table::new(&fic_results)
                .with(Style::psql())
                .with(Panel::header("Federated Identity Credentials"))
        );
        self.save_to_csv(&fic_results, "federated_identity_credentials.csv")?;

        // Print out the results as a table with a RedirectUris header
        let uri_results = self.check_redirect_uris().await?;
        println!(
            "{}\n",
            Table::new(&uri_results)
                .with(Style::psql())
                .with(Panel::header("Redirect URIs"))
        );
        self.save_to_csv(&uri_results, "redirect_uris.csv")?;

        // Print out the results as a table with a loginUrls header
        let login_results = self.check_login_urls().await?;
        println!(
            "{}\n",
            Table::new(&login_results)
                .with(Style::psql())
                .with(Panel::header("Service Principal loginUrls"))
        );
        self.save_to_csv(&login_results, "login_urls.csv")?;

        // Print out the results as a table with a ReplyUrls header
        let reply_results = self.check_reply_urls().await?;
        println!(
            "{}\n",
            Table::new(&reply_results)
                .with(Style::psql())
                .with(Panel::header("Reply URLs"))
        );
        self.save_to_csv(&reply_results, "reply_urls.csv")?;

        Ok(())
    }

    /// Checks if a domain returns NXDOMAIN (does not exist in DNS)
    async fn is_domain_nxdomain(&self, domain: &str) -> Result<bool, CirroGraphError> {
        // Check cache first
        {
            let mut cache = self.nxdomain_cache.lock().unwrap();
            if let Some(&cached_result) = cache.get(domain) {
                debug!("NXDOMAIN cache hit for domain: {}", domain);
                return Ok(cached_result);
            }
        }

        let resolver = TokioResolver::builder_tokio()
            .map_err(|e| {
                CirroGraphError::DnsResolverError(format!(
                    "Failed to create DNS resolver: {}",
                    e.to_string()
                ))
            })?
            .build();

        let result = match resolver.lookup_ip(domain).await {
            Ok(response) => {
                debug!(
                    "DNS A record found for: {} (found {} records)",
                    domain,
                    response.iter().count()
                );
                false // Domain exists, not NXDOMAIN
            }
            Err(e) => {
                if e.is_nx_domain() {
                    debug!("Domain does not exist (NXDOMAIN): {}", domain);
                    true // Domain is NXDOMAIN
                } else {
                    debug!("DNS lookup error for {}: {}", domain, e.to_string());
                    false // Assume domain exists but has other DNS issues
                }
            }
        };

        // Cache the result
        {
            let mut cache = self.nxdomain_cache.lock().unwrap();
            cache.put(domain.to_string(), result);
        }

        Ok(result)
    }

    /// Checks if a domain is available for registration using WHOIS
    async fn check_domain_availability(&self, domain: &str) -> Result<bool, CirroGraphError> {
        // Check cache first
        {
            let mut cache = self.availability_cache.lock().unwrap();
            if let Some(&cached_result) = cache.get(domain) {
                debug!("Availability cache hit for domain: {}", domain);
                return Ok(cached_result);
            }
        }

        let result = match self.whois_client.check_domain(domain).await {
            Ok(info) => {
                if info.available == Some(true) {
                    debug!("Domain {} is available for registration", domain);
                    true
                } else {
                    debug!("Domain {} is registered", domain);
                    false
                }
            }
            Err(e) => {
                warn!("WHOIS lookup failed for {}: {}", domain, e);
                return Err(CirroGraphError::WhoisError(format!(
                    "WHOIS lookup failed for {}: {}",
                    domain, e
                )));
            }
        };

        // Cache the result
        {
            let mut cache = self.availability_cache.lock().unwrap();
            cache.put(domain.to_string(), result);
        }

        Ok(result)
    }

    /// Gets cache statistics for debugging
    pub fn get_cache_stats(&self) -> (usize, usize, usize, usize) {
        let nxdomain_cache = self.nxdomain_cache.lock().unwrap();
        let availability_cache = self.availability_cache.lock().unwrap();

        (
            nxdomain_cache.len(),
            nxdomain_cache.cap().get(),
            availability_cache.len(),
            availability_cache.cap().get(),
        )
    }

    /// Checks VerifiedDomains for missing or invalid DNS records
    async fn check_verified_domains(&self) -> Result<Vec<DomainState>, CirroGraphError> {
        info!("Checking VerifiedDomains for missing or invalid DNS records...");

        let mut domain_states: Vec<DomainState> = Vec::new();

        // Keep a hashmap of root domain results to avoid duplicates
        let mut root_domain_cache: std::collections::HashMap<String, (bool, bool)> =
            std::collections::HashMap::new();

        let query_str = r#"
            MATCH (vd:VerifiedDomain)
            RETURN vd.name AS domain
        "#;

        let mut query_result = self.graph.execute(query(query_str)).await.map_err(|e| {
            CirroGraphError::DatabaseError(format!("Failed to execute query: {}", e.to_string()))
        })?;

        while let Ok(Some(row)) = query_result.next().await {
            let domain: String = row.get("domain").unwrap_or_default();

            // Parse root domain
            let root_domain = psl::domain_str(&domain);
            if let Some(root_domain_str) = root_domain {
                if root_domain_str.is_empty() {
                    debug!(
                        "Could not parse root domain from '{}', got empty string",
                        domain
                    );
                    continue;
                }

                // Check if we already have results for this root domain
                let (root_is_nxdomain, root_is_available) =
                    if let Some(&cached) = root_domain_cache.get(root_domain_str) {
                        debug!("Using cached root domain results for: {}", root_domain_str);
                        cached
                    } else {
                        debug!("Checking root domain: {}", root_domain_str);

                        // Check if the root domain is NXDOMAIN
                        let root_nxdomain = self
                            .is_domain_nxdomain(root_domain_str)
                            .await
                            .unwrap_or(false);

                        // Check if the root domain is available for registration
                        let root_available = if root_nxdomain {
                            match self.check_domain_availability(root_domain_str).await {
                                Ok(available) => available,
                                Err(e) => {
                                    warn!(
                                        "WHOIS availability check failed for {}: {}",
                                        root_domain_str, e
                                    );
                                    false
                                }
                            }
                        } else {
                            false
                        };

                        // Cache the results
                        root_domain_cache
                            .insert(root_domain_str.to_string(), (root_nxdomain, root_available));
                        (root_nxdomain, root_available)
                    };

                // Check the subdomain itself
                let is_nxdomain = self.is_domain_nxdomain(&domain).await.unwrap_or(false);
                let is_available = if is_nxdomain {
                    match self.check_domain_availability(&domain).await {
                        Ok(available) => available,
                        Err(e) => {
                            debug!("WHOIS availability check failed for {}: {}", domain, e);
                            false
                        }
                    }
                } else {
                    false
                };

                let domain_state = DomainState {
                    domain: domain.clone(),
                    is_nxdomain,
                    is_available,
                    root_domain: root_domain_str.to_string(),
                    root_is_nxdomain,
                    root_is_available,
                };

                domain_states.push(domain_state);
            } else {
                info!(
                    "Could not parse root domain from '{}', skipping DNS check",
                    domain
                );
                continue;
            }
        }

        Ok(domain_states)
    }

    /// Checks FederatedIdentityCredentials for missing or invalid DNS records
    async fn check_federated_identity_credentials(
        &self,
    ) -> Result<Vec<FederatedIdentityCredentialState>, CirroGraphError> {
        info!("Checking FederatedIdentityCredentials for missing or invalid DNS records...");

        let mut fic_states: Vec<FederatedIdentityCredentialState> = Vec::new();

        let query_str = r#"
            MATCH (a:GraphApplication)-[:FEDERATED_CREDENTIAL]->(n:FederatedIdentityCredential) 
            RETURN a.appId, a.displayName, a.publisherDomain, n.issuer
        "#;

        let mut query_result = self.graph.execute(query(query_str)).await.map_err(|e| {
            CirroGraphError::DatabaseError(format!("Failed to execute query: {}", e.to_string()))
        })?;

        while let Ok(Some(row)) = query_result.next().await {
            let app_id: String = row.get("a.appId").unwrap_or_default();
            let display_name: String = row.get("a.displayName").unwrap_or_default();
            let publisher_domain: String = row.get("a.publisherDomain").unwrap_or_default();
            let issuer: String = row.get("n.issuer").unwrap_or_default();

            // URL parse the issuer to extract the domain
            let issuer_domain = match url::Url::parse(&issuer) {
                Ok(url) => url.host_str().unwrap_or("").to_string(),
                Err(_) => {
                    warn!(
                        "Could not parse issuer URL '{}' for app '{}', skipping DNS check",
                        issuer, display_name
                    );
                    continue;
                }
            };

            // Check if nxdomain
            let is_nxdomain = self
                .is_domain_nxdomain(&issuer_domain)
                .await
                .unwrap_or(false);

            let is_available = if is_nxdomain {
                let root_domain = psl::domain_str(&issuer_domain);
                if let Some(root_domain_str) = root_domain {
                    if root_domain_str.is_empty() {
                        debug!(
                            "Could not parse root domain from '{}', got empty string",
                            issuer_domain
                        );
                        false
                    } else {
                        match self.check_domain_availability(root_domain_str).await {
                            Ok(available) => available,
                            Err(e) => {
                                debug!(
                                    "WHOIS availability check failed for {}: {}",
                                    root_domain_str, e
                                );
                                false
                            }
                        }
                    }
                } else {
                    debug!(
                        "Could not parse root domain from '{}', setting availability to false",
                        issuer_domain
                    );
                    false
                }
            } else {
                false
            };

            let fic_state = FederatedIdentityCredentialState {
                app_id,
                display_name,
                publisher_domain,
                issuer: issuer_domain,
                is_nxdomain,
                is_available,
            };
            fic_states.push(fic_state);
        }

        Ok(fic_states)
    }

    /// Checks RedirectUris for missing or invalid DNS records
    async fn check_redirect_uris(&self) -> Result<Vec<RedirectUriState>, CirroGraphError> {
        info!("Checking RedirectUris for missing or invalid DNS records...");

        let mut uri_states: Vec<RedirectUriState> = Vec::new();

        let query_str = r#"
            MATCH (s:GraphApplication)
            UNWIND coalesce(s.redirectUris, []) AS redirectUri
            RETURN
            s.appId AS appId,
            s.publisherDomain AS publisherDomain,
            s.signInAudience AS signInAudience,
            s.displayName AS displayName,
            redirectUri
        "#;

        let mut query_result = self.graph.execute(query(query_str)).await.map_err(|e| {
            CirroGraphError::DatabaseError(format!("Failed to execute query: {}", e.to_string()))
        })?;

        while let Ok(Some(row)) = query_result.next().await {
            let app_id: String = row.get("appId").unwrap_or_default();
            let display_name: String = row.get("displayName").unwrap_or_default();
            let publisher_domain: String = row.get("publisherDomain").unwrap_or_default();
            let sign_in_audience: String = row.get("signInAudience").unwrap_or_default();
            let redirect_uri: String = row.get("redirectUri").unwrap_or_default();

            // redirect_uri should start with http or https
            if !redirect_uri.starts_with("http://") && !redirect_uri.starts_with("https://") {
                debug!(
                    "Redirect URI '{}' for app '{}' does not start with http:// or https://, skipping DNS check",
                    redirect_uri, display_name
                );
                continue;
            }

            // URL parse the redirect_uri to extract the domain
            let uri_domain = match url::Url::parse(&redirect_uri) {
                Ok(url) => url.host_str().unwrap_or("").to_string(),
                Err(_) => {
                    warn!(
                        "Could not parse redirect URI '{}' for app '{}', skipping DNS check",
                        redirect_uri, display_name
                    );
                    continue;
                }
            };

            // If the domain is "localhost", set is_nxdomain and is_available to false
            if uri_domain == "localhost" {
                let uri_state = RedirectUriState {
                    app_id,
                    display_name,
                    publisher_domain,
                    sign_in_audience,
                    redirect_uri,
                    is_nxdomain: false,
                    is_available: false,
                };
                uri_states.push(uri_state);
                continue;
            }

            let is_nxdomain = self.is_domain_nxdomain(&uri_domain).await.unwrap_or(false);
            let is_available = if is_nxdomain {
                let root_domain = psl::domain_str(&uri_domain);

                if let Some(root_domain_str) = root_domain {
                    if root_domain_str.is_empty() {
                        debug!(
                            "Could not parse root domain from '{}', got empty string",
                            uri_domain
                        );
                        false
                    } else {
                        match self.check_domain_availability(root_domain_str).await {
                            Ok(available) => available,
                            Err(e) => {
                                debug!(
                                    "WHOIS availability check failed for {}: {}",
                                    root_domain_str, e
                                );
                                false
                            }
                        }
                    }
                } else {
                    debug!(
                        "Could not parse root domain from '{}', setting availability to false",
                        uri_domain
                    );
                    false
                }
            } else {
                false
            };

            let uri_state = RedirectUriState {
                app_id,
                display_name,
                publisher_domain,
                sign_in_audience,
                redirect_uri,
                is_nxdomain,
                is_available,
            };
            uri_states.push(uri_state);
        }

        Ok(uri_states)
    }

    /// Checks service principal loginUrls for missing or invalid DNS records
    async fn check_login_urls(&self) -> Result<Vec<LoginUrlState>, CirroGraphError> {
        info!("Checking Service Principal loginUrls for missing or invalid DNS records...");

        let mut uri_states: Vec<LoginUrlState> = Vec::new();

        let query_str = r#"
            MATCH (s:GraphServicePrincipal)
            WHERE s.loginUrl IS NOT NULL AND s.loginUrl <> ""
            RETURN
            s.appId AS appId,
            s.displayName AS displayName,
            s.loginUrl AS loginUrl
        "#;

        let mut query_result = self.graph.execute(query(query_str)).await.map_err(|e| {
            CirroGraphError::DatabaseError(format!("Failed to execute query: {}", e.to_string()))
        })?;

        while let Ok(Some(row)) = query_result.next().await {
            let app_id: String = row.get("appId").unwrap_or_default();
            let display_name: String = row.get("displayName").unwrap_or_default();
            let login_url: String = row.get("loginUrl").unwrap_or_default();

            // URL parse the login_url to extract the domain
            let url_domain = match url::Url::parse(&login_url) {
                Ok(url) => url.host_str().unwrap_or("").to_string(),
                Err(_) => {
                    warn!(
                        "Could not parse login URL '{}' for app '{}', skipping DNS check",
                        login_url, display_name
                    );
                    continue;
                }
            };

            // If the domain is "localhost", set is_nxdomain and is_available to false
            if url_domain == "localhost" {
                let uri_state = LoginUrlState {
                    app_id,
                    display_name,
                    login_url,
                    is_nxdomain: false,
                    is_available: false,
                };
                uri_states.push(uri_state);
                continue;
            }

            let is_nxdomain = self.is_domain_nxdomain(&url_domain).await.unwrap_or(false);
            let is_available = if is_nxdomain {
                let root_domain = psl::domain_str(&url_domain);
                if let Some(root_domain_str) = root_domain {
                    if root_domain_str.is_empty() {
                        debug!(
                            "Could not parse root domain from '{}', got empty string",
                            url_domain
                        );
                        false
                    } else {
                        match self.check_domain_availability(root_domain_str).await {
                            Ok(available) => available,
                            Err(e) => {
                                debug!(
                                    "WHOIS availability check failed for {}: {}",
                                    root_domain_str, e
                                );
                                false
                            }
                        }
                    }
                } else {
                    debug!(
                        "Could not parse root domain from '{}', setting availability to false",
                        url_domain
                    );
                    false
                }
            } else {
                false
            };

            let uri_state = LoginUrlState {
                app_id,
                display_name,
                login_url,
                is_nxdomain,
                is_available,
            };
            uri_states.push(uri_state);
        }
        Ok(uri_states)
    }

    /// Checks ReplyUrls for missing or invalid DNS records
    pub async fn check_reply_urls(&self) -> Result<Vec<ReplyUrlState>, CirroGraphError> {
        info!("Checking ReplyUrls for missing or invalid DNS records...");

        let mut reply_url_states: Vec<ReplyUrlState> = Vec::new();

        let query_str = r#"
            MATCH (s:GraphServicePrincipal)
            WHERE NOT s.appOwnerOrganizationId IN ["f8cdef31-a31e-4b4a-93e4-5f571e91255a", "72f988bf-86f1-41af-91ab-2d7cd011db47"]
            UNWIND s.replyUrls AS replyUrl
            RETURN
            s.id AS id,
            s.appId AS appId,
            s.appOwnerOrganizationId AS appOwnerOrganizationId,
            s.displayName AS displayName,
            replyUrl
        "#;

        let mut query_result = self.graph.execute(query(query_str)).await.map_err(|e| {
            CirroGraphError::DatabaseError(format!("Failed to execute query: {}", e.to_string()))
        })?;

        while let Ok(Some(row)) = query_result.next().await {
            let app_id: String = row.get("appId").unwrap_or_default();
            let app_org_id: String = row.get("appOwnerOrganizationId").unwrap_or_default();
            let display_name: String = row.get("displayName").unwrap_or_default();
            let reply_url: String = row.get("replyUrl").unwrap_or_default();

            // reply_url should start with http or https
            if !reply_url.starts_with("http://") && !reply_url.starts_with("https://") {
                debug!(
                    "Reply URL '{}' for app '{}' does not start with http:// or https://, skipping DNS check",
                    reply_url, display_name
                );
                continue;
            }

            // URL parse the reply_url to extract the domain
            let url_domain = match url::Url::parse(&reply_url) {
                Ok(url) => url.host_str().unwrap_or("").to_string(),
                Err(_) => {
                    debug!(
                        "Could not parse reply URL '{}' for app '{}', skipping DNS check",
                        reply_url, display_name
                    );
                    continue;
                }
            };

            // If the domain is "localhost", set is_nxdomain and is_available to false
            if url_domain == "localhost" {
                let reply_state = ReplyUrlState {
                    app_id,
                    app_org_id,
                    display_name,
                    reply_url,
                    is_nxdomain: false,
                    is_available: false,
                };
                reply_url_states.push(reply_state);
                continue;
            }

            let is_nxdomain = self.is_domain_nxdomain(&url_domain).await.unwrap_or(false);
            let is_available = if is_nxdomain {
                let root_domain = psl::domain_str(&url_domain);

                if let Some(root_domain_str) = root_domain {
                    if root_domain_str.is_empty() {
                        debug!(
                            "Could not parse root domain from '{}', got empty string",
                            url_domain
                        );
                        false
                    } else {
                        match self.check_domain_availability(root_domain_str).await {
                            Ok(available) => available,
                            Err(e) => {
                                debug!(
                                    "WHOIS availability check failed for {}: {}",
                                    root_domain_str, e
                                );
                                false
                            }
                        }
                    }
                } else {
                    debug!(
                        "Could not parse root domain from '{}', setting availability to false",
                        url_domain
                    );
                    false
                }
            } else {
                false
            };

            let reply_state = ReplyUrlState {
                app_id,
                app_org_id,
                display_name,
                reply_url,
                is_nxdomain,
                is_available,
            };
            reply_url_states.push(reply_state);
        }

        Ok(reply_url_states)
    }
}
