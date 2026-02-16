use log::{debug, error};
use rusqlite::Connection;

use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct DataMessage {
    pub table: String,
    pub id: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct ArmResourceMessage {
    pub id: String,
    pub sub_id: String,
    pub rg_id: String,
    pub resource_type: String,
    pub data: serde_json::Value,
}

pub enum DBWriteMessage {
    Data(DataMessage),
    DataBatch(Vec<DataMessage>),
    ArmResource(ArmResourceMessage),
    ArmResourceBatch(Vec<ArmResourceMessage>),
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
    pub fn run_writer(&self, mut receiver: mpsc::Receiver<DBWriteMessage>) {
        // Define all possible table names
        let table_names = vec![
            "applications",
            "administrativeUnits",
            "devices",
            "directoryRoles",
            "eligibleRoleAssignments",
            "groups",
            "managementGroupEntities",
            "organization",
            "policies",
            "roleAssignments",
            "servicePrincipals",
            "subscriptions",
            "tenants",
            "users",
        ];

        const WRITE_BATCH_SIZE: usize = 1000;

        let init_db = || -> Result<Connection, rusqlite::Error> {
            let conn = Connection::open(&self.db_path)?;

            let _ = conn.busy_timeout(Duration::from_secs(30));

            if let Err(e) = conn.execute_batch(
                "PRAGMA journal_mode=WAL;
                 PRAGMA synchronous=NORMAL;
                 PRAGMA automatic_index=true;
                 PRAGMA temp_store=MEMORY;",
            ) {
                error!("Failed to set database PRAGMAs: {}", e);
            }

            for table_name in &table_names {
                let create_table_sql = format!(
                    "CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, data BLOB)",
                    table_name
                );
                if let Err(e) = conn.execute(&create_table_sql, []) {
                    error!("Failed to create table {}: {}", table_name, e);
                }
            }

            if let Err(e) = conn.execute(
                "CREATE TABLE IF NOT EXISTS resources (id TEXT PRIMARY KEY, sub_id TEXT, rg_id TEXT, resource_type TEXT, data BLOB)",
                [],
            ) {
                error!("Failed to create resources table: {}", e);
            }

            Ok(conn)
        };

        // Connection will be initialized when the first message arrives
        let mut conn_option: Option<Connection> = None;
        let mut in_transaction = false;
        let mut pending_writes: usize = 0;

        // Process incoming messages
        while let Some(message) = receiver.blocking_recv() {
            if conn_option.is_none() {
                debug!(
                    "Received first message, initializing database: {:?}",
                    self.db_path
                );
                match init_db() {
                    Ok(conn) => {
                        conn_option = Some(conn);
                        debug!("Database initialized successfully");
                    }
                    Err(e) => {
                        error!("Failed to open database: {}", e);
                        continue;
                    }
                }
            }

            let conn = match conn_option.as_mut() {
                Some(c) => c,
                None => {
                    error!("Database connection unavailable, skipping message");
                    continue;
                }
            };

            let write_succeeded = match message {
                DBWriteMessage::Data(message) => {
                    let DataMessage { table, id, data } = message;

                    if !in_transaction {
                        if let Err(e) = conn.execute_batch("BEGIN IMMEDIATE") {
                            error!("Failed to begin transaction: {}", e);
                            continue;
                        }
                        in_transaction = true;
                    }

                    // Insert or replace the data
                    let insert_sql = format!("REPLACE INTO {} (id, data) VALUES (?, ?)", table);
                    match conn.prepare_cached(&insert_sql).and_then(|mut stmt| {
                        stmt.execute(rusqlite::params![id.to_lowercase(), data.to_string()])
                    }) {
                        Ok(_) => {}
                        Err(e) => {
                            error!(
                                "Failed to write to table: {}, id: {}. Error: {}",
                                table, id, e
                            );
                            continue;
                        }
                    }

                    true
                }
                DBWriteMessage::DataBatch(rows) => {
                    if rows.is_empty() {
                        continue;
                    }

                    let table = rows[0].table.clone();

                    if !in_transaction {
                        if let Err(e) = conn.execute_batch("BEGIN IMMEDIATE") {
                            error!("Failed to begin transaction: {}", e);
                            continue;
                        }
                        in_transaction = true;
                    }

                    let insert_sql = format!("REPLACE INTO {} (id, data) VALUES (?, ?)", table);
                    let mut stmt = match conn.prepare_cached(&insert_sql) {
                        Ok(stmt) => stmt,
                        Err(e) => {
                            error!("Failed to prepare statement for table {}: {}", table, e);
                            continue;
                        }
                    };

                    let mut succeeded = 0usize;
                    for row in rows {
                        if row.table != table {
                            error!(
                                "Mixed table names in DataBatch (expected {}, got {})",
                                table, row.table
                            );
                            continue;
                        }

                        match stmt.execute(rusqlite::params![
                            row.id.to_lowercase(),
                            row.data.to_string()
                        ]) {
                            Ok(_) => succeeded += 1,
                            Err(e) => {
                                error!(
                                    "Failed to write to table: {}, id: {}. Error: {}",
                                    table, row.id, e
                                );
                            }
                        }
                    }

                    if succeeded == 0 {
                        continue;
                    }

                    pending_writes += succeeded.saturating_sub(1);
                    true
                }
                DBWriteMessage::ArmResource(message) => {
                    let ArmResourceMessage {
                        id,
                        sub_id,
                        rg_id,
                        resource_type,
                        data,
                    } = message;

                    if !in_transaction {
                        if let Err(e) = conn.execute_batch("BEGIN IMMEDIATE") {
                            error!("Failed to begin transaction: {}", e);
                            continue;
                        }
                        in_transaction = true;
                    }

                    let insert_sql = "REPLACE INTO resources (id, sub_id, rg_id, resource_type, data) VALUES (?, ?, ?, ?, ?)";
                    match conn.prepare_cached(insert_sql).and_then(|mut stmt| {
                        stmt.execute(rusqlite::params![
                            id.to_lowercase(),
                            sub_id.to_lowercase(),
                            rg_id.to_lowercase(),
                            resource_type.to_lowercase(),
                            data.to_string(),
                        ])
                    }) {
                        Ok(_) => {}
                        Err(e) => {
                            error!(
                                "Failed to write ARM resource: {}, sub_id: {}, rg_id: {}, resource_type: {}. Error: {}",
                                id, sub_id, rg_id, resource_type, e
                            );
                            continue;
                        }
                    }

                    true
                }
                DBWriteMessage::ArmResourceBatch(rows) => {
                    if rows.is_empty() {
                        continue;
                    }

                    if !in_transaction {
                        if let Err(e) = conn.execute_batch("BEGIN IMMEDIATE") {
                            error!("Failed to begin transaction: {}", e);
                            continue;
                        }
                        in_transaction = true;
                    }

                    let insert_sql = "REPLACE INTO resources (id, sub_id, rg_id, resource_type, data) VALUES (?, ?, ?, ?, ?)";
                    let mut stmt = match conn.prepare_cached(insert_sql) {
                        Ok(stmt) => stmt,
                        Err(e) => {
                            error!("Failed to prepare resource insert statement: {}", e);
                            continue;
                        }
                    };

                    let mut succeeded = 0usize;
                    for row in rows {
                        match stmt.execute(rusqlite::params![
                            row.id.to_lowercase(),
                            row.sub_id.to_lowercase(),
                            row.rg_id.to_lowercase(),
                            row.resource_type.to_lowercase(),
                            row.data.to_string(),
                        ]) {
                            Ok(_) => succeeded += 1,
                            Err(e) => {
                                error!(
                                    "Failed to write ARM resource: {}, sub_id: {}, rg_id: {}, resource_type: {}. Error: {}",
                                    row.id, row.sub_id, row.rg_id, row.resource_type, e
                                );
                            }
                        }
                    }

                    if succeeded == 0 {
                        continue;
                    }

                    pending_writes += succeeded.saturating_sub(1);
                    true
                }
                DBWriteMessage::Shutdown => {
                    debug!("Received shutdown message, closing database connection");

                    if in_transaction {
                        if let Err(e) = conn.execute_batch("COMMIT") {
                            error!("Failed to commit transaction during shutdown: {}", e);
                        }
                        in_transaction = false;
                    }

                    // Finalize any pending work and close the connection properly
                    if let Some(conn) = conn_option.as_ref() {
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
            };

            if write_succeeded {
                pending_writes += 1;

                if pending_writes >= WRITE_BATCH_SIZE && in_transaction {
                    if let Err(e) = conn.execute_batch("COMMIT") {
                        error!("Failed to commit write batch: {}", e);
                    }
                    in_transaction = false;
                    pending_writes = 0;
                }
            }
        }

        if in_transaction {
            if let Some(conn) = conn_option.as_ref() {
                if let Err(e) = conn.execute_batch("COMMIT") {
                    error!("Failed to commit final transaction: {}", e);
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
