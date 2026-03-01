use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::{Pool, Postgres, Column, Row};

#[derive(Debug, Clone)]
pub enum DatabaseEngine {
    PostgreSQL,
    MySQL,
    SQLite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub affected_rows: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub engine: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub struct DatabaseConnection {
    pool: Pool<Postgres>,
    engine: DatabaseEngine,
}

impl DatabaseConnection {
    pub async fn connect(config: ConnectionConfig) -> Result<Self, String> {
        let connection_string = match config.engine.as_str() {
            "postgresql" | "postgres" => {
                let host = config.host.unwrap_or_else(|| "localhost".to_string());
                let port = config.port.unwrap_or(5432);
                let user = config.username.unwrap_or_else(|| "postgres".to_string());
                let pass = config.password.unwrap_or_default();
                format!("postgres://{}:{}@{}:{}/{}", user, pass, host, port, config.database)
            }
            "mysql" => {
                let host = config.host.unwrap_or_else(|| "localhost".to_string());
                let port = config.port.unwrap_or(3306);
                let user = config.username.unwrap_or_else(|| "root".to_string());
                let pass = config.password.unwrap_or_default();
                format!("mysql://{}:{}@{}:{}/{}", user, pass, host, port, config.database)
            }
            "sqlite" => {
                format!("{}", config.database)
            }
            _ => return Err(format!("Unsupported database engine: {}", config.engine)),
        };

        let pool = sqlx::PgPool::connect(&connection_string)
            .await
            .map_err(|e| format!("Failed to connect to PostgreSQL: {}", e))?;

        Ok(DatabaseConnection {
            pool,
            engine: DatabaseEngine::PostgreSQL,
        })
    }

    pub async fn query(&self, sql: &str) -> Result<QueryResult, String> {
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Query failed: {}", e))?;

        let mut columns = Vec::new();
        let mut result_rows = Vec::new();
        let row_count = rows.len();

        if row_count > 0 {
            for col in rows[0].columns() {
                columns.push(col.name().to_string());
            }

            for row in &rows {
                let mut row_data = Vec::new();
                for i in 0..columns.len() {
                    let value: serde_json::Value = match row.try_get::<String, _>(i) {
                        Ok(v) => serde_json::Value::String(v),
                        Err(_) => {
                            match row.try_get::<i64, _>(i) {
                                Ok(v) => serde_json::Value::Number(v.into()),
                                Err(_) => {
                                    match row.try_get::<f64, _>(i) {
                                        Ok(v) => serde_json::json!(v),
                                        Err(_) => serde_json::Value::Null,
                                    }
                                }
                            }
                        }
                    };
                    row_data.push(value);
                }
                result_rows.push(row_data);
            }
        }

        Ok(QueryResult {
            columns,
            rows: result_rows,
            affected_rows: row_count as u64,
        })
    }

    pub async fn execute(&self, sql: &str) -> Result<u64, String> {
        let result = sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Execute failed: {}", e))?;

        Ok(result.rows_affected())
    }

    pub async fn close(&self) -> Result<(), String> {
        self.pool.close().await;
        Ok(())
    }
}

pub struct DbModule {
    connections: Arc<Mutex<HashMap<String, DatabaseConnection>>>,
}

impl DbModule {
    pub fn new() -> Self {
        DbModule {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn connect(&self, name: String, config: ConnectionConfig) -> Result<String, String> {
        let conn = DatabaseConnection::connect(config).await?;
        let mut connections = self.connections.lock().await;
        connections.insert(name.clone(), conn);
        Ok(format!("Connected to database as '{}'", name))
    }

    pub async fn query(&self, connection_name: &str, sql: &str) -> Result<QueryResult, String> {
        let connections = self.connections.lock().await;
        let conn = connections.get(connection_name)
            .ok_or_else(|| format!("Connection '{}' not found", connection_name))?;
        conn.query(sql).await
    }

    pub async fn execute(&self, connection_name: &str, sql: &str) -> Result<u64, String> {
        let connections = self.connections.lock().await;
        let conn = connections.get(connection_name)
            .ok_or_else(|| format!("Connection '{}' not found", connection_name))?;
        conn.execute(sql).await
    }

    pub async fn disconnect(&self, name: &str) -> Result<String, String> {
        let mut connections = self.connections.lock().await;
        if connections.remove(name).is_some() {
            Ok(format!("Disconnected from '{}'", name))
        } else {
            Err(format!("Connection '{}' not found", name))
        }
    }

    pub async fn list_connections(&self) -> Vec<String> {
        let connections = self.connections.lock().await;
        connections.keys().cloned().collect()
    }
}

impl Default for DbModule {
    fn default() -> Self {
        Self::new()
    }
}
