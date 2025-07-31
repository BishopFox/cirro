use log::{debug, error};
use rusqlite::Connection;

use std::path::PathBuf;
use tokio::sync::mpsc;

pub enum DBWriteMessage {
    Data {
        table: String,
        id: String,
        data: serde_json::Value,
    },
    ArmResource {
        id: String,
        sub_id: String,
        rg_id: String,
        resource_type: String,
        data: serde_json::Value,
    },
    Enrichment {
        module: String,
        resource_id: String,
        data: serde_json::Value,
    },
    /// Shutdown message to close the database connection
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct SqliteDb {
    pub db_path: PathBuf,
}

impl SqliteDb {
    pub fn new(db_path: PathBuf) -> Self {
        SqliteDb { db_path }
    }

    /// Gets a connection to the SQLite database
    pub async fn get_connection(&self) -> Result<Connection, rusqlite::Error> {
        Connection::open(&self.db_path)
    }

    /// Runs the database writer loop
    /// This will initialize the database connection when the first message arrives
    /// and will handle all incoming messages until a shutdown message is received.
    pub async fn run_writer(&self, mut receiver: mpsc::UnboundedReceiver<DBWriteMessage>) {
        // Define all possible table names
        let table_names = vec![
            "applications",
            "administrativeUnits",
            "devices",
            "directoryRoles",
            "groups",
            "organization",
            "policies",
            "roleAssignments",
            "servicePrincipals",
            "subscriptions",
            "tenants",
            "users",
        ];

        // Connection will be initialized when the first message arrives
        let mut conn_option: Option<Connection> = None;

        // Process incoming messages
        while let Some(message) = receiver.recv().await {
            match message {
                DBWriteMessage::Data { table, id, data } => {
                    // Initialize database if this is the first message
                    if conn_option.is_none() {
                        debug!(
                            "Received first message, initializing database: {:?}",
                            self.db_path
                        );

                        // Open the database connection
                        match Connection::open(&self.db_path) {
                            Ok(new_conn) => {
                                // Set database PRAGMAs for performance
                                if let Err(e) = new_conn.execute_batch(
                                    "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA automatic_index = true;",
                                ) {
                                    error!("Failed to set database PRAGMAs: {}", e);
                                    // Continue anyway, as these are just performance optimizations
                                }

                                // Create standard tables if they do not exist
                                for table_name in &table_names {
                                    let create_table_sql = format!(
                                        "CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, data BLOB)",
                                        table_name
                                    );
                                    if let Err(e) = new_conn.execute(&create_table_sql, []) {
                                        error!("Failed to create table {}: {}", table_name, e);
                                        // Continue with other tables
                                    }
                                }

                                // Create the resource table with its special schema
                                if let Err(e) = new_conn.execute(
                                    "CREATE TABLE IF NOT EXISTS resources (id TEXT PRIMARY KEY, sub_id TEXT, rg_id TEXT, resource_type TEXT, data BLOB)",
                                    [],
                                ) {
                                    error!("Failed to create resources table: {}", e);
                                    // Continue anyway
                                }

                                // Create the enrichment table
                                if let Err(e) = new_conn.execute(
                                    "CREATE TABLE IF NOT EXISTS enrichments (module TEXT, resource_id TEXT, data BLOB, PRIMARY KEY (module, resource_id))",
                                    [],
                                ) {
                                    error!("Failed to create enrichments table: {}", e);
                                    // Continue anyway
                                }

                                conn_option = Some(new_conn);
                                debug!("Database initialized successfully");
                            }
                            Err(e) => {
                                error!("Failed to open database: {}", e);
                                // Skip this message since we couldn't open the database
                                continue;
                            }
                        };
                    }

                    // At this point we should have a valid connection
                    let conn = match &conn_option {
                        Some(c) => c,
                        None => {
                            error!("Database connection unavailable, skipping message");
                            continue;
                        }
                    };

                    // Insert or replace the data
                    let insert_sql =
                        format!("INSERT OR REPLACE INTO {} (id, data) VALUES (?, ?)", table);
                    match conn.execute(&insert_sql, &[&id.to_lowercase(), &data.to_string()]) {
                        Ok(_) => {}
                        Err(e) => {
                            error!(
                                "Failed to write to table: {}, id: {}. Error: {}",
                                table, id, e
                            );
                            continue;
                        }
                    }
                }
                DBWriteMessage::ArmResource {
                    id,
                    sub_id,
                    rg_id,
                    resource_type,
                    data,
                } => {
                    let insert_sql = "REPLACE INTO resources (id, sub_id, rg_id, resource_type, data) VALUES (?, ?, ?, ?, ?)";
                    let conn = match &conn_option {
                        Some(c) => c,
                        None => {
                            error!("Database connection unavailable, skipping message");
                            continue;
                        }
                    };
                    match conn.execute(
                        insert_sql,
                        &[
                            &id.to_lowercase(),
                            &sub_id.to_lowercase(),
                            &rg_id.to_lowercase(),
                            &resource_type.to_lowercase(),
                            &data.to_string(),
                        ],
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            error!(
                                "Failed to write ARM resource: {}, sub_id: {}, rg_id: {}, resource_type: {}. Error: {}",
                                id, sub_id, rg_id, resource_type, e
                            );
                            continue;
                        }
                    }
                }
                DBWriteMessage::Enrichment {
                    module,
                    resource_id,
                    data,
                } => {
                    let insert_sql =
                        "REPLACE INTO enrichments (module, resource_id, data) VALUES (?, ?, ?)";
                    let conn = match &conn_option {
                        Some(c) => c,
                        None => {
                            // Set up new connection
                            conn_option = Some(Connection::open(&self.db_path).unwrap());
                            match conn_option.as_ref() {
                                Some(c) => c,
                                None => {
                                    error!("Failed to open database connection for enrichment");
                                    continue;
                                }
                            }
                        }
                    };
                    match conn.execute(insert_sql, &[&module, &resource_id, &data.to_string()]) {
                        Ok(_) => {}
                        Err(e) => {
                            error!(
                                "Failed to write enrichment: module: {}, resource_id: {}. Error: {}",
                                module, resource_id, e
                            );
                            continue;
                        }
                    }
                }
                DBWriteMessage::Shutdown => {
                    debug!("Received shutdown message, closing database connection");
                    // Finalize any pending work and close the connection properly
                    if let Some(conn) = &conn_option {
                        if let Err(e) = conn.execute("PRAGMA optimize", []) {
                            debug!("Failed to run PRAGMA optimize: {}", e);
                        }
                        debug!("Database shutdown complete");
                    } else {
                        debug!("No active database connection to shut down");
                    }
                    // Break out of the loop to terminate the writer
                    break;
                }
            }
        }
    }

    /// Runs a read query against the database
    pub fn run_query(&self, query: &str) -> Result<Vec<serde_json::Value>, rusqlite::Error> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(serde_json::from_str(&data).unwrap_or_default())
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}
