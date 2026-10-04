use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
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
    /// Full connection string. When set, it takes precedence over the
    /// individual host/port/user fields (§22: works for all engines; for
    /// non-SQL engines it is usually a broker/cluster URL).
    #[serde(default)]
    pub connection_string: Option<String>,
    /// Desired pool size. `None` = engine default (see [`PoolConfig::default`]).
    #[serde(default)]
    pub max_connections: Option<u32>,
    /// Connect/query timeout in seconds. `None` = 30s.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    /// Whether to require TLS. `None` = engine default (plaintext locally).
    #[serde(default)]
    pub tls: Option<bool>,
    /// Free-form engine options (e.g. `sslmode`, `keyspace`, `region`).
    #[serde(default)]
    pub options: HashMap<String, String>,
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

// ============================================================================
// §22 — Database abstraction (SQL + engines + pool/tx/prepared/builder/
// migrations + optional ORM) · §58 — NoSQL ecosystem
// ============================================================================

/// Unified error type for the whole `lexicon-db` abstraction layer.
///
/// `String` errors on the legacy [`DatabaseConnection`] API are preserved
/// for backwards compatibility; new APIs return this structured error.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Connection / pool failure.
    #[error("connection failed: {0}")]
    Connection(String),
    /// Query / execute failure.
    #[error("query failed: {0}")]
    Query(String),
    /// Feature not supported by the selected engine.
    #[error("unsupported by engine {engine}: {feature}")]
    Unsupported {
        engine: &'static str,
        feature: &'static str,
    },
    /// Migration failure (ordering conflict, failed step, ...).
    #[error("migration failed: {0}")]
    Migration(String),
    /// Operation timed out.
    #[error("operation timed out after {0:?}")]
    Timeout(Duration),
    /// Bounded buffer full — caller must retry (backpressure signal).
    #[error("backpressure: buffer full, retry later")]
    Backpressure,
    /// Anything else (serde, poisoned state, ...).
    #[error("{0}")]
    Other(String),
}

/// Full engine catalogue for §22 (SQL) + §58 (NoSQL / search / KV).
///
/// # Guarantees (per engine)
///
/// - **Transactions:** only the SQL engines (`Postgres`, `MySql`, `Sqlite`)
///   offer ACID transactions. [`Transaction`] in this file is an in-memory
///   staging buffer: it gives atomicity of the *staged batch* on `commit`
///   (all-or-nothing against the recorded log) but **no** durability or
///   isolation by itself. For real ACID semantics use the engine natively
///   (via `sqlx` for the SQL engines). NoSQL engines expose at best
///   single-key atomicity (documented per stub).
/// - **Concurrency:** every public type in this module is `Send + Sync`
///   (`Arc` + `tokio::sync::Mutex/RwLock` internally) and can be shared
///   across tasks. The SQL path additionally relies on the `sqlx` pool.
/// - **Cancellation:** all `async` methods are cancellation-safe — dropping
///   the future never corrupts shared state (locks are held only across
///   synchronous critical sections, never across network I/O).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Engine {
    /// PostgreSQL — full ACID, prepared statements, `sqlx::PgPool`.
    Postgres,
    /// MySQL/MariaDB — ACID (InnoDB), prepared statements.
    MySql,
    /// SQLite — ACID, single-writer, prepared statements.
    Sqlite,
    /// Redis — single-command atomicity; no multi-key transactions here.
    /// See [`RedisCommands`] / [`InMemoryRedis`].
    Redis,
    /// MongoDB — single-document atomicity; see [`DocumentStore`].
    Mongo,
    /// DynamoDB — single-item atomicity, eventually consistent reads.
    Dynamo,
    /// Cassandra — tunable consistency, lightweight transactions only.
    Cassandra,
    /// Elasticsearch — near-real-time search index, no transactions.
    Elastic,
    /// Generic in-process KV (tests, caches, embedded use).
    /// See [`KvStore`] / [`InMemoryKvStore`].
    KvMemory,
}

impl Engine {
    /// Canonical lowercase name (matches `ConnectionConfig::engine` strings).
    pub fn as_str(self) -> &'static str {
        match self {
            Engine::Postgres => "postgres",
            Engine::MySql => "mysql",
            Engine::Sqlite => "sqlite",
            Engine::Redis => "redis",
            Engine::Mongo => "mongo",
            Engine::Dynamo => "dynamo",
            Engine::Cassandra => "cassandra",
            Engine::Elastic => "elastic",
            Engine::KvMemory => "kv",
        }
    }

    /// Whether this engine speaks SQL.
    pub fn is_sql(self) -> bool {
        matches!(self, Engine::Postgres | Engine::MySql | Engine::Sqlite)
    }

    /// Whether real (ACID) transactions are available natively.
    pub fn supports_transactions(self) -> bool {
        self.is_sql()
    }

    /// Whether server-side prepared statements are available.
    pub fn supports_prepared_statements(self) -> bool {
        self.is_sql()
    }

    /// Default TCP port (embedded engines return `None`).
    pub fn default_port(self) -> Option<u16> {
        match self {
            Engine::Postgres => Some(5432),
            Engine::MySql => Some(3306),
            Engine::Sqlite => None,
            Engine::Redis => Some(6379),
            Engine::Mongo => Some(27017),
            Engine::Dynamo => None, // HTTP API, region-scoped
            Engine::Cassandra => Some(9042),
            Engine::Elastic => Some(9200),
            Engine::KvMemory => None,
        }
    }
}

impl std::str::FromStr for Engine {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "postgres" | "postgresql" => Ok(Engine::Postgres),
            "mysql" | "mariadb" => Ok(Engine::MySql),
            "sqlite" => Ok(Engine::Sqlite),
            "redis" => Ok(Engine::Redis),
            "mongo" | "mongodb" => Ok(Engine::Mongo),
            "dynamo" | "dynamodb" => Ok(Engine::Dynamo),
            "cassandra" => Ok(Engine::Cassandra),
            "elastic" | "elasticsearch" => Ok(Engine::Elastic),
            "kv" | "memory" | "kvmemory" => Ok(Engine::KvMemory),
            other => Err(format!("unknown engine: {other}")),
        }
    }
}

/// Connection-pool configuration (§22: pool).
///
/// # Guarantees
/// - `Send + Sync + Clone`; sharing a pool across tasks is safe.
/// - Validation is total: [`PoolConfig::validate`] rejects `0` connections
///   or `min > max` before any resource is allocated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Maximum concurrent connections. Default 10.
    pub max_connections: u32,
    /// Minimum idle connections to keep warm. Default 1.
    pub min_connections: u32,
    /// Connect timeout in seconds. Default 30.
    pub connect_timeout_secs: u64,
    /// Idle-connection eviction timeout in seconds. Default 600.
    pub idle_timeout_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        PoolConfig {
            max_connections: 10,
            min_connections: 1,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
        }
    }
}

impl PoolConfig {
    /// Checks the invariant `0 < min <= max` and non-zero timeouts.
    pub fn validate(&self) -> Result<(), DbError> {
        if self.max_connections == 0 {
            return Err(DbError::Other("max_connections must be > 0".into()));
        }
        if self.min_connections == 0 {
            return Err(DbError::Other("min_connections must be > 0".into()));
        }
        if self.min_connections > self.max_connections {
            return Err(DbError::Other(
                "min_connections must be <= max_connections".into(),
            ));
        }
        if self.connect_timeout_secs == 0 {
            return Err(DbError::Other("connect_timeout_secs must be > 0".into()));
        }
        Ok(())
    }

    /// Connect timeout as a [`Duration`].
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connect_timeout_secs)
    }
}

impl ConnectionConfig {
    /// Minimal constructor; optional fields default to `None`/empty.
    pub fn new(engine: impl Into<String>, database: impl Into<String>) -> Self {
        ConnectionConfig {
            engine: engine.into(),
            host: None,
            port: None,
            database: database.into(),
            username: None,
            password: None,
            connection_string: None,
            max_connections: None,
            timeout_secs: None,
            tls: None,
            options: HashMap::new(),
        }
    }

    /// Parses [`ConnectionConfig::engine`] into an [`Engine`].
    /// Unknown names fall back to `Postgres` for backwards compatibility
    /// with the legacy [`DatabaseConnection`] (which is Postgres-backed).
    pub fn engine_kind(&self) -> Engine {
        self.engine.parse().unwrap_or(Engine::Postgres)
    }

    /// Derives the effective [`PoolConfig`] from the optional overrides.
    pub fn pool_config(&self) -> PoolConfig {
        let mut pool = PoolConfig::default();
        if let Some(max) = self.max_connections {
            pool.max_connections = max;
        }
        if let Some(t) = self.timeout_secs {
            pool.connect_timeout_secs = t;
        }
        pool
    }

    /// Effective operation timeout.
    pub fn effective_timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs.unwrap_or(30))
    }
}

/// A parameterised statement (§22: prepared).
///
/// Placeholders follow the `$1, $2, ...` (Postgres/SQLite) convention in
/// generated SQL; [`QueryBuilder::build_prepared`] numbers them in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedStatement {
    /// SQL text with `$N` placeholders.
    pub sql: String,
    /// Parameters in placeholder order (JSON-encoded values).
    pub params: Vec<serde_json::Value>,
}

impl PreparedStatement {
    /// Creates an (initially parameter-free) prepared statement.
    pub fn new(sql: impl Into<String>) -> Self {
        PreparedStatement {
            sql: sql.into(),
            params: Vec::new(),
        }
    }

    /// Appends one bound parameter, returning `self` for chaining.
    pub fn bind(mut self, value: serde_json::Value) -> Self {
        self.params.push(value);
        self
    }
}

/// Fluent `SELECT` builder (§22: builder).
///
/// ```rust
/// use lexicon_db::module::{QueryBuilder, PreparedStatement};
/// let stmt: PreparedStatement = QueryBuilder::new("users")
///     .select(["id", "name"])
///     .where_clause("age > $1", serde_json::json!(18))
///     .limit(10)
///     .build_prepared();
/// assert!(stmt.sql.contains("SELECT"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
    table: String,
    columns: Vec<String>,
    filters: Vec<String>,
    params: Vec<serde_json::Value>,
    limit: Option<u64>,
    offset: Option<u64>,
}

impl QueryBuilder {
    /// Starts a query against `table`.
    pub fn new(table: impl Into<String>) -> Self {
        QueryBuilder {
            table: table.into(),
            ..Default::default()
        }
    }

    /// Column list (`SELECT a, b`). Empty = `SELECT *`.
    pub fn select(mut self, columns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.columns.extend(columns.into_iter().map(Into::into));
        self
    }

    /// `WHERE` condition with one bound parameter (Lex `where` clause).
    /// The condition should reference the next `$N` placeholder; use
    /// [`QueryBuilder::param`] for conditions without placeholders.
    pub fn where_clause(mut self, condition: impl Into<String>, param: serde_json::Value) -> Self {
        self.filters.push(condition.into());
        self.params.push(param);
        self
    }

    /// Raw `WHERE` condition fragment without binding a parameter.
    pub fn filter(mut self, condition: impl Into<String>) -> Self {
        self.filters.push(condition.into());
        self
    }

    /// Pushes a bound parameter without adding a condition fragment.
    pub fn param(mut self, value: serde_json::Value) -> Self {
        self.params.push(value);
        self
    }

    /// Maximum rows to return (`LIMIT`).
    pub fn limit(mut self, n: u64) -> Self {
        self.limit = Some(n);
        self
    }

    /// Rows to skip (`OFFSET`).
    pub fn offset(mut self, n: u64) -> Self {
        self.offset = Some(n);
        self
    }

    /// Borrowed view of the bound parameters.
    pub fn params(&self) -> &[serde_json::Value] {
        &self.params
    }

    /// Builds `(sql, params)`.
    pub fn build(self) -> (String, Vec<serde_json::Value>) {
        let cols = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };
        let mut sql = format!("SELECT {} FROM {}", cols, self.table);
        if !self.filters.is_empty() {
            sql.push_str(&format!(" WHERE {}", self.filters.join(" AND ")));
        }
        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {}", n));
        }
        if let Some(n) = self.offset {
            sql.push_str(&format!(" OFFSET {}", n));
        }
        (sql, self.params)
    }

    /// Builds a [`PreparedStatement`].
    pub fn build_prepared(self) -> PreparedStatement {
        let (sql, params) = self.build();
        PreparedStatement { sql, params }
    }
}

/// Portable database abstraction (§22: `query`, `execute`, `prepared`).
///
/// # Guarantees
/// - Implementors must be `Send + Sync` so handles can move across tasks.
/// - `prepared` must bind parameters without string interpolation (no SQL
///   injection by construction) on engines with
///   [`Engine::supports_prepared_statements`]; other engines must return
///   [`DbError::Unsupported`] rather than interpolating.
/// - Methods are cancellation-safe (see [`Engine`] docs).
pub trait Database: Send + Sync {
    /// Runs a read query and returns rows.
    #[allow(async_fn_in_trait)]
    async fn query(&self, sql: &str) -> Result<QueryResult, DbError>;
    /// Runs a write statement and returns affected rows.
    #[allow(async_fn_in_trait)]
    async fn execute(&self, sql: &str) -> Result<u64, DbError>;
    /// Runs a parameterised statement without interpolation.
    #[allow(async_fn_in_trait)]
    async fn prepared(&self, stmt: &PreparedStatement) -> Result<QueryResult, DbError>;
    /// Which engine backs this handle.
    fn engine(&self) -> Engine;
}

impl Database for DatabaseConnection {
    async fn query(&self, sql: &str) -> Result<QueryResult, DbError> {
        DatabaseConnection::query(self, sql).await.map_err(DbError::Query)
    }

    async fn execute(&self, sql: &str) -> Result<u64, DbError> {
        DatabaseConnection::execute(self, sql).await.map_err(DbError::Query)
    }

    async fn prepared(&self, stmt: &PreparedStatement) -> Result<QueryResult, DbError> {
        // Postgres-backed pool: bind every param as text/JSON; engines that
        // cannot bind safely must refuse instead of interpolating.
        let mut q = sqlx::query(&stmt.sql);
        for p in &stmt.params {
            let s = match p {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            q = q.bind(s);
        }
        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;
        let mut columns = Vec::new();
        let mut result_rows = Vec::new();
        if let Some(first) = rows.first() {
            for col in first.columns() {
                columns.push(col.name().to_string());
            }
            for row in &rows {
                let mut row_data = Vec::new();
                for i in 0..columns.len() {
                    let value: serde_json::Value = match row.try_get::<String, _>(i) {
                        Ok(v) => serde_json::Value::String(v),
                        Err(_) => match row.try_get::<i64, _>(i) {
                            Ok(v) => serde_json::Value::Number(v.into()),
                            Err(_) => match row.try_get::<f64, _>(i) {
                                Ok(v) => serde_json::json!(v),
                                Err(_) => serde_json::Value::Null,
                            },
                        },
                    };
                    row_data.push(value);
                }
                result_rows.push(row_data);
            }
        }
        Ok(QueryResult {
            columns,
            rows: result_rows,
            affected_rows: rows.len() as u64,
        })
    }

    fn engine(&self) -> Engine {
        Engine::Postgres
    }
}

/// Lifecycle of a [`Transaction`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionState {
    /// Accepting staged statements.
    Active,
    /// `commit` was called; staged batch was handed off exactly once.
    Committed,
    /// `rollback` was called; staged batch was discarded.
    RolledBack,
}

static TRANSACTION_IDS: AtomicU64 = AtomicU64::new(1);

/// Staged unit of work (§22: tx).
///
/// # Guarantees
/// - **Atomicity of the hand-off:** `commit` consumes `self` and returns the
///   staged batch exactly once; `rollback` consumes `self` and discards it.
///   There is no half-committed state observable after either call.
/// - **Scope:** this is a *staging buffer*, not a live engine transaction —
///   it provides no isolation/durability against concurrent writers. Use the
///   engine natively (ACID on SQL engines) when those are required.
/// - **Concurrency:** `Send` but not `Sync` by design (single owner stages,
///   then commits); share the [`DatabaseConnection`], not the transaction.
/// - **Cancellation:** `commit`/`rollback` are synchronous and infallible
///   w.r.t. cancellation (no `.await` inside).
#[derive(Debug)]
pub struct Transaction {
    /// Unique id for tracing/logging.
    pub id: u64,
    state: TransactionState,
    staged: Vec<String>,
}

impl Transaction {
    /// Begins a new (empty, active) transaction.
    pub fn begin() -> Self {
        Transaction {
            id: TRANSACTION_IDS.fetch_add(1, Ordering::Relaxed),
            state: TransactionState::Active,
            staged: Vec::new(),
        }
    }

    /// Stages one statement. Fails if no longer active.
    pub fn stage(&mut self, sql: impl Into<String>) -> Result<(), DbError> {
        if self.state != TransactionState::Active {
            return Err(DbError::Other(format!(
                "transaction {} is not active ({:?})",
                self.id, self.state
            )));
        }
        self.staged.push(sql.into());
        Ok(())
    }

    /// Current lifecycle state.
    pub fn state(&self) -> TransactionState {
        self.state
    }

    /// Number of staged statements.
    pub fn len(&self) -> usize {
        self.staged.len()
    }

    /// Whether any statement is staged.
    pub fn is_empty(&self) -> bool {
        self.staged.is_empty()
    }

    /// Commits: hands off the staged batch exactly once.
    pub fn commit(mut self) -> Result<Vec<String>, DbError> {
        if self.state != TransactionState::Active {
            return Err(DbError::Other(format!(
                "transaction {} cannot commit from state {:?}",
                self.id, self.state
            )));
        }
        self.state = TransactionState::Committed;
        Ok(std::mem::take(&mut self.staged))
    }

    /// Rolls back: discards the staged batch.
    pub fn rollback(mut self) -> Result<(), DbError> {
        if self.state != TransactionState::Active {
            return Err(DbError::Other(format!(
                "transaction {} cannot rollback from state {:?}",
                self.id, self.state
            )));
        }
        self.state = TransactionState::RolledBack;
        self.staged.clear();
        Ok(())
    }
}

impl DatabaseConnection {
    /// Opens a client-side staged transaction (see [`Transaction`] for the
    /// exact guarantees — not a live engine transaction).
    pub fn begin(&self) -> Transaction {
        Transaction::begin()
    }
}

/// One versioned schema step (§22: migrations).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    /// Monotonic version; runners apply in ascending order.
    pub version: u64,
    /// Human-readable label.
    pub description: String,
    /// Forward DDL/DML.
    pub up_sql: String,
    /// Backward DDL/DML.
    pub down_sql: String,
}

impl Migration {
    /// Creates one migration step.
    pub fn new(
        version: u64,
        description: impl Into<String>,
        up_sql: impl Into<String>,
        down_sql: impl Into<String>,
    ) -> Self {
        Migration {
            version,
            description: description.into(),
            up_sql: up_sql.into(),
            down_sql: down_sql.into(),
        }
    }
}

/// Ordered, idempotent migration runner (§22: migrations).
///
/// # Guarantees
/// - **Ordered:** plans are always emitted in ascending `version` order,
///   regardless of registration order.
/// - **Idempotent:** versions already in `applied` are skipped, so running
///   the same plan twice is a no-op the second time.
/// - **Validation:** [`MigrationRunner::validate`] rejects duplicate
///   versions before anything is planned.
/// - Pure in-memory bookkeeping (`Send + Sync`); executing the SQL against a
///   real engine is the caller's job (pair with [`Database::execute`]).
#[derive(Debug, Default)]
pub struct MigrationRunner {
    migrations: BTreeMap<u64, Migration>,
    applied: HashSet<u64>,
}

impl MigrationRunner {
    /// Empty runner with nothing applied.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one migration (replaces any same-version entry).
    pub fn register(&mut self, migration: Migration) {
        self.migrations.insert(migration.version, migration);
    }

    /// Rejects duplicate registration mistakes. (Since `register`
    /// overwrites, duplicates can only arise from caller-side double
    /// registration of *different* payloads — `validate` also asserts the
    /// map is non-degenerate; callers building from a `Vec` should check
    /// lengths first via [`MigrationRunner::validate_vec`].)
    pub fn validate(&self) -> Result<(), DbError> {
        // BTreeMap keys are unique by construction; the meaningful check is
        // that every entry round-trips its own key.
        for (version, m) in &self.migrations {
            if m.version != *version {
                return Err(DbError::Migration(format!(
                    "migration key {version} != payload version {}",
                    m.version
                )));
            }
        }
        Ok(())
    }

    /// Helper: validates a `Vec` *before* registering (detects duplicates).
    pub fn validate_vec(migrations: &[Migration]) -> Result<(), DbError> {
        let mut seen = HashSet::new();
        for m in migrations {
            if !seen.insert(m.version) {
                return Err(DbError::Migration(format!(
                    "duplicate migration version {}",
                    m.version
                )));
            }
        }
        Ok(())
    }

    /// Versions already applied (sorted).
    pub fn applied_versions(&self) -> Vec<u64> {
        let mut v: Vec<u64> = self.applied.iter().copied().collect();
        v.sort_unstable();
        v
    }

    /// Pending migrations in ascending order (idempotency: applied skipped).
    pub fn pending(&self) -> Vec<&Migration> {
        self.migrations
            .values()
            .filter(|m| !self.applied.contains(&m.version))
            .collect()
    }

    /// `up_sql` of every pending migration, in order. Empty = up to date.
    pub fn plan_up(&self) -> Vec<(u64, String)> {
        self.pending()
            .into_iter()
            .map(|m| (m.version, m.up_sql.clone()))
            .collect()
    }

    /// Records versions as applied (e.g. after successful execution).
    pub fn mark_applied(&mut self, versions: impl IntoIterator<Item = u64>) {
        self.applied.extend(versions);
    }

    /// Records a version as reverted (becomes pending again).
    pub fn mark_reverted(&mut self, version: u64) -> bool {
        self.applied.remove(&version)
    }

    /// `down_sql` for rolling back the single newest applied version.
    pub fn plan_down_last(&self) -> Option<(u64, String)> {
        self.applied.iter().max().and_then(|v| {
            self.migrations
                .get(v)
                .map(|m| (m.version, m.down_sql.clone()))
        })
    }
}

// ----------------------------------------------------------------------------
// §58 — NoSQL: generic KV, documents, Redis
// ----------------------------------------------------------------------------

/// Generic key-value store (§58: kv genérico).
///
/// # Guarantees
/// - Implementors are `Send + Sync` (safe to share across tasks).
/// - Single-key operations are atomic; **no** multi-key transactions.
/// - Methods are cancellation-safe (no observable half-write on cancel).
pub trait KvStore: Send + Sync {
    /// Reads a key (`None` = missing).
    #[allow(async_fn_in_trait)]
    async fn get(&self, key: &str) -> Result<Option<String>, DbError>;
    /// Writes (creates or overwrites) a key.
    #[allow(async_fn_in_trait)]
    async fn set(&self, key: &str, value: &str) -> Result<(), DbError>;
    /// Deletes a key; returns whether it existed.
    #[allow(async_fn_in_trait)]
    async fn del(&self, key: &str) -> Result<bool, DbError>;
    /// Whether the key exists.
    #[allow(async_fn_in_trait)]
    async fn exists(&self, key: &str) -> Result<bool, DbError> {
        Ok(self.get(key).await?.is_some())
    }
}

/// In-process KV store: the [`Engine::KvMemory`] backend and the default
/// for tests. Backed by `tokio::sync::RwLock<HashMap>` — `Send + Sync`,
/// multi-reader/single-writer; the lock is held only across synchronous
/// map operations, so all methods are cancellation-safe.
#[derive(Debug, Default)]
pub struct InMemoryKvStore {
    map: RwLock<HashMap<String, String>>,
}

impl InMemoryKvStore {
    /// Empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of keys (test/debug helper).
    pub async fn len(&self) -> usize {
        self.map.read().await.len()
    }

    /// Whether the store is empty.
    pub async fn is_empty(&self) -> bool {
        self.map.read().await.is_empty()
    }
}

impl KvStore for InMemoryKvStore {
    async fn get(&self, key: &str) -> Result<Option<String>, DbError> {
        Ok(self.map.read().await.get(key).cloned())
    }

    async fn set(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.map
            .write()
            .await
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn del(&self, key: &str) -> Result<bool, DbError> {
        Ok(self.map.write().await.remove(key).is_some())
    }

    async fn exists(&self, key: &str) -> Result<bool, DbError> {
        Ok(self.map.read().await.contains_key(key))
    }
}

/// Document-store abstraction (§58: Mongo/Dynamo-style documents).
///
/// # Guarantees
/// - Single-document operations are atomic; no cross-document transactions.
/// - `Send + Sync`; cancellation-safe (same locking discipline as
///   [`InMemoryKvStore`]).
pub trait DocumentStore: Send + Sync {
    /// Inserts a JSON document into `collection`, returning its id.
    #[allow(async_fn_in_trait)]
    async fn insert(
        &self,
        collection: &str,
        document: serde_json::Value,
    ) -> Result<String, DbError>;
    /// Fetches one document by id (`None` = missing).
    #[allow(async_fn_in_trait)]
    async fn find_by_id(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<serde_json::Value>, DbError>;
    /// Deletes one document by id; returns whether it existed.
    #[allow(async_fn_in_trait)]
    async fn delete(&self, collection: &str, id: &str) -> Result<bool, DbError>;
}

/// In-memory [`DocumentStore`] stub (per-collection `HashMap<id, doc>`).
/// Ids are monotonically assigned (`doc-1`, `doc-2`, ...).
#[derive(Debug, Default)]
pub struct InMemoryDocumentStore {
    collections: RwLock<HashMap<String, HashMap<String, serde_json::Value>>>,
    next_id: AtomicU64,
}

impl InMemoryDocumentStore {
    /// Empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl DocumentStore for InMemoryDocumentStore {
    async fn insert(
        &self,
        collection: &str,
        document: serde_json::Value,
    ) -> Result<String, DbError> {
        let id = format!("doc-{}", self.next_id.fetch_add(1, Ordering::Relaxed) + 1);
        let mut cols = self.collections.write().await;
        cols.entry(collection.to_string())
            .or_default()
            .insert(id.clone(), document);
        Ok(id)
    }

    async fn find_by_id(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<serde_json::Value>, DbError> {
        Ok(self
            .collections
            .read()
            .await
            .get(collection)
            .and_then(|c| c.get(id))
            .cloned())
    }

    async fn delete(&self, collection: &str, id: &str) -> Result<bool, DbError> {
        let mut cols = self.collections.write().await;
        Ok(cols
            .get_mut(collection)
            .map(|c| c.remove(id).is_some())
            .unwrap_or(false))
    }
}

/// Minimal Redis command surface (§58: Redis).
///
/// # Guarantees
/// - Single-command atomicity only (mirrors real Redis semantics).
/// - `publish` here records onto an in-process log and returns the number
///   of recorded deliveries (stub transport — no real Pub/Sub fan-out).
pub trait RedisCommands: Send + Sync {
    /// `GET key`.
    #[allow(async_fn_in_trait)]
    async fn redis_get(&self, key: &str) -> Result<Option<String>, DbError>;
    /// `SET key value`.
    #[allow(async_fn_in_trait)]
    async fn redis_set(&self, key: &str, value: &str) -> Result<(), DbError>;
    /// `DEL key`.
    #[allow(async_fn_in_trait)]
    async fn redis_del(&self, key: &str) -> Result<bool, DbError>;
    /// `PUBLISH channel message` — returns deliveries recorded by the stub.
    #[allow(async_fn_in_trait)]
    async fn publish(&self, channel: &str, message: &str) -> Result<u64, DbError>;
}

/// In-memory [`RedisCommands`] stub over a KV map plus a publish log.
/// `Send + Sync`, cancellation-safe (same locking discipline as
/// [`InMemoryKvStore`]).
#[derive(Debug, Default)]
pub struct InMemoryRedis {
    kv: InMemoryKvStore,
    published: Mutex<Vec<(String, String)>>,
}

impl InMemoryRedis {
    /// Empty stub.
    pub fn new() -> Self {
        Self::default()
    }

    /// All `(channel, message)` pairs recorded via [`RedisCommands::publish`].
    pub async fn published_log(&self) -> Vec<(String, String)> {
        self.published.lock().await.clone()
    }
}

impl KvStore for InMemoryRedis {
    async fn get(&self, key: &str) -> Result<Option<String>, DbError> {
        self.kv.get(key).await
    }
    async fn set(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.kv.set(key, value).await
    }
    async fn del(&self, key: &str) -> Result<bool, DbError> {
        self.kv.del(key).await
    }
    async fn exists(&self, key: &str) -> Result<bool, DbError> {
        self.kv.exists(key).await
    }
}

impl RedisCommands for InMemoryRedis {
    async fn redis_get(&self, key: &str) -> Result<Option<String>, DbError> {
        self.kv.get(key).await
    }
    async fn redis_set(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.kv.set(key, value).await
    }
    async fn redis_del(&self, key: &str) -> Result<bool, DbError> {
        self.kv.del(key).await
    }
    async fn publish(&self, channel: &str, message: &str) -> Result<u64, DbError> {
        self.published
            .lock()
            .await
            .push((channel.to_string(), message.to_string()));
        Ok(1)
    }
}

// ----------------------------------------------------------------------------
// §22 — Optional ORM: Entity trait + manual-derive example
// ----------------------------------------------------------------------------

/// Minimal ORM mapping (§22: ORM opcional).
///
/// A `#[derive(Entity)]` macro would generate exactly the `table()` +
/// `from_row()`/`to_row()` trio below from struct fields (field name =
/// column name, `Option<T>` = nullable column). Until the macro exists,
/// implement this trait by hand as shown in [`UserEntity`].
pub trait Entity: Sized {
    /// Table backing this entity.
    fn table() -> &'static str;
    /// Builds an entity from a column-name → value map.
    /// Missing/non-nullable or mistyped columns must yield `Err`.
    fn from_row(row: &HashMap<String, serde_json::Value>) -> Result<Self, DbError>;
    /// Serialises the entity back to a column map (round-trips `from_row`).
    fn to_row(&self) -> HashMap<String, serde_json::Value>;

    /// Convenience: builds one entity per row of a [`QueryResult`] in
    /// column order, skipping rows that fail to convert.
    fn from_query_result(result: &QueryResult) -> Vec<Self> {
        result
            .rows
            .iter()
            .filter_map(|values| {
                let map: HashMap<String, serde_json::Value> = result
                    .columns
                    .iter()
                    .cloned()
                    .zip(values.iter().cloned())
                    .collect();
                Self::from_row(&map).ok()
            })
            .collect()
    }
}

/// Example entity with a *manual* [`Entity`] implementation — i.e. exactly
/// what `#[derive(Entity)]` must generate for
/// `struct UserEntity { id: i64, name: String, email: Option<String> }`
/// over `table = "users"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserEntity {
    /// Primary key (`users.id`, NOT NULL).
    pub id: i64,
    /// Display name (`users.name`, NOT NULL).
    pub name: String,
    /// Contact address (`users.email`, nullable).
    pub email: Option<String>,
}

impl Entity for UserEntity {
    fn table() -> &'static str {
        "users"
    }

    fn from_row(row: &HashMap<String, serde_json::Value>) -> Result<Self, DbError> {
        let id = row
            .get("id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| DbError::Other("missing/invalid non-null column `id`".into()))?;
        let name = row
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DbError::Other("missing/invalid non-null column `name`".into()))?
            .to_string();
        let email = match row.get("email") {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(s)) => Some(s.clone()),
            Some(other) => {
                return Err(DbError::Other(format!(
                    "invalid nullable column `email`: {other}"
                )))
            }
        };
        Ok(UserEntity { id, name, email })
    }

    fn to_row(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), serde_json::json!(self.id));
        map.insert("name".to_string(), serde_json::json!(self.name));
        map.insert(
            "email".to_string(),
            self.email
                .clone()
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        );
        map
    }
}
