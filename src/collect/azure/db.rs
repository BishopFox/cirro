use log::{debug, error};
use rusqlite::Connection;

use std::collections::HashMap;
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
    /// Save (upsert) the current pagination state for a graph resource type
    SaveState {
        resource_type: String,
        next_uri: String,
        total_collected: usize,
        elapsed_secs: f64,
    },
    /// Clear the pagination state for a graph resource type (enumeration complete)
    ClearState {
        resource_type: String,
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
    pub fn run_writer(&self, mut receiver: mpsc::Receiver<DBWriteMessage>) {
        // Define all possible table names
        let table_names = vec![
            "applications",
            "administrativeUnits",
            "conditionalAccessPolicies",
            "devices",
            "directoryRoles",
            "eligibleArmRBAC",
            "eligibleRoleAssignments",
            "groups",
            "managementGroupEntities",
            "namedLocations",
            "organization",
            "oauth2PermissionGrants",
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
                    "CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, data TEXT)",
                    table_name
                );
                if let Err(e) = conn.execute(&create_table_sql, []) {
                    error!("Failed to create table {}: {}", table_name, e);
                }
            }

            if let Err(e) = conn.execute(
                "CREATE TABLE IF NOT EXISTS resources (id TEXT PRIMARY KEY, sub_id TEXT, rg_id TEXT, resource_type TEXT, data TEXT)",
                [],
            ) {
                error!("Failed to create resources table: {}", e);
            }

            if let Err(e) = conn.execute(
                "CREATE TABLE IF NOT EXISTS _state (resource_type TEXT PRIMARY KEY, next_uri TEXT NOT NULL, total_collected INTEGER NOT NULL DEFAULT 0, elapsed_secs REAL NOT NULL DEFAULT 0.0)",
                [],
            ) {
                error!("Failed to create _state table: {}", e);
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
                DBWriteMessage::SaveState {
                    resource_type,
                    next_uri,
                    total_collected,
                    elapsed_secs,
                } => {
                    if !in_transaction {
                        if let Err(e) = conn.execute_batch("BEGIN IMMEDIATE") {
                            error!("Failed to begin transaction: {}", e);
                            continue;
                        }
                        in_transaction = true;
                    }

                    match conn.execute(
                        "INSERT OR REPLACE INTO _state (resource_type, next_uri, total_collected, elapsed_secs) VALUES (?, ?, ?, ?)",
                        rusqlite::params![resource_type, next_uri, total_collected as i64, elapsed_secs],
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to save state for {}: {}", resource_type, e);
                            continue;
                        }
                    }

                    true
                }
                DBWriteMessage::ClearState { resource_type } => {
                    if !in_transaction {
                        if let Err(e) = conn.execute_batch("BEGIN IMMEDIATE") {
                            error!("Failed to begin transaction: {}", e);
                            continue;
                        }
                        in_transaction = true;
                    }

                    match conn.execute(
                        "DELETE FROM _state WHERE resource_type = ?",
                        rusqlite::params![resource_type],
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to clear state for {}: {}", resource_type, e);
                            continue;
                        }
                    }

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

    /// Reads saved enumeration state from the _state table, optionally filtered by prefix.
    /// Returns a map of resource_type -> (next_uri, total_collected, elapsed_secs).
    pub fn read_state(
        &self,
        prefix: Option<&str>,
    ) -> Result<HashMap<String, (String, usize, f64)>, rusqlite::Error> {
        let conn = Connection::open(&self.db_path)?;
        // Table may not exist yet on a fresh DB
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_state'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);
        if !exists {
            return Ok(HashMap::new());
        }

        let mut state = HashMap::new();
        if let Some(pfx) = prefix {
            let like_pattern = format!("{}%", pfx);
            let mut stmt = conn.prepare(
                "SELECT resource_type, next_uri, total_collected, elapsed_secs FROM _state WHERE resource_type LIKE ?",
            )?;
            let rows = stmt.query_map(rusqlite::params![like_pattern], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)? as usize,
                    row.get::<_, f64>(3)?,
                ))
            })?;
            for row in rows {
                let (rt, uri, count, elapsed) = row?;
                state.insert(rt, (uri, count, elapsed));
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT resource_type, next_uri, total_collected, elapsed_secs FROM _state",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)? as usize,
                    row.get::<_, f64>(3)?,
                ))
            })?;
            for row in rows {
                let (rt, uri, count, elapsed) = row?;
                state.insert(rt, (uri, count, elapsed));
            }
        }
        Ok(state)
    }

    /// Reads all saved graph pagination state (no prefix filter).
    pub fn read_graph_state(
        &self,
    ) -> Result<HashMap<String, (String, usize, f64)>, rusqlite::Error> {
        self.read_state(None)
    }

    /// Clears saved enumeration state, optionally scoped by prefix.
    pub fn clear_state(&self, prefix: Option<&str>) -> Result<(), rusqlite::Error> {
        let conn = Connection::open(&self.db_path)?;
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_state'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);
        if exists {
            if let Some(pfx) = prefix {
                let like_pattern = format!("{}%", pfx);
                conn.execute(
                    "DELETE FROM _state WHERE resource_type LIKE ?",
                    rusqlite::params![like_pattern],
                )?;
            } else {
                conn.execute("DELETE FROM _state", [])?;
            }
        }
        Ok(())
    }

    /// Clears all saved graph pagination state from the _state table.
    pub fn clear_graph_state(&self) -> Result<(), rusqlite::Error> {
        self.clear_state(None)
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
