use mysql_async::prelude::*;
use mysql_async::Pool;
use std::env;

/// Configuration for the MySQL connection, read from environment variables.
/// Supports Northflank's environment variables injected when a MySQL addon is linked to a secret group.
#[derive(Default)]
pub struct MySqlConfig {
    pub uri: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
}

impl MySqlConfig {
    /// Load configuration from environment variables.
    ///
    /// Northflank injects these variables when you link a MySQL addon to a secret group:
    /// - `DB_MYSQL_JDBC_URI`: Full JDBC connection string (e.g., `jdbc:mysql://host:port/database`)
    /// - `DB_HOST`, `DB_PORT`, `DB_USERNAME`, `DB_PASSWORD`, `DB_DATABASE`: Individual connection details
    pub fn from_env() -> Self {
        Self {
            uri: env::var("DB_MYSQL_JDBC_URI").ok(),
            host: env::var("DB_HOST").ok(),
            port: env::var("DB_PORT")
                .ok()
                .and_then(|p| p.parse().ok()),
            username: env::var("DB_USERNAME").ok(),
            password: env::var("DB_PASSWORD").ok(),
            database: env::var("DB_DATABASE").ok(),
        }
    }

    /// Build a connection string from the configuration.
    /// Prefers `DB_MYSQL_JDBC_URI` if available, converting it from JDBC format; otherwise constructs from individual parts.
    pub fn connection_string(&self) -> Option<String> {
        // Prefer the full JDBC URI if provided (Northflank default)
        if let Some(ref uri) = self.uri {
            return Some(jdbc_uri_to_mysql(uri));
        }

        // Construct from individual parts
        let host = self.host.as_ref()?;
        let port = self.port?;
        let username = self.username.as_ref()?;
        let password = self.password.as_ref()?;
        let database = self.database.as_ref()?;

        Some(format!(
            "mysql://{}:{}@{}:{}/{}",
            username, password, host, port, database
        ))
    }
}

/// Initialize a MySQL connection pool from environment variables.
/// Returns `None` if no MySQL configuration is found (graceful degradation for local dev).
pub async fn init_pool() -> Option<Pool> {
    let conn_str = MySqlConfig::from_env().connection_string()
        .or_else(|| {
            println!("No MySQL configuration found. Skipping database connection.");
            None
        })?;
    init_pool_from_conn_str(&conn_str).await
}

/// Create a MySQL connection pool from a specific connection string.
pub async fn init_pool_from_conn_str(conn_str: &str) -> Option<Pool> {
    println!("Connecting to MySQL: {}", host_masked(conn_str));
    let opts = conn_str.parse::<mysql_async::Opts>().ok()?;
    let pool = Pool::new(opts);
    pool.get_conn().await.ok()?;
    println!("MySQL connection established successfully.");
    Some(pool)
}

/// Create a MySQL connection pool connected via Unix socket.
/// Skips the initial test connection to avoid blocking the single-threaded runtime;
/// the caller should use `mysql_local::wait_for_pool` to verify readiness.
pub fn init_pool_from_socket(socket_path: &str) -> Option<Pool> {
    println!("Connecting to MySQL via socket: {}", socket_path);
    let conn_str = socket_conn_str(socket_path);
    let opts: mysql_async::Opts = conn_str.parse().ok()?;
    Some(Pool::new(opts))
}

/// Build a mysql_async connection string for a Unix socket path.
fn socket_conn_str(socket_path: &str) -> String {
    let encoded = urlencoding(socket_path);
    format!("mysql://root@localhost/?socket={}", encoded)
}

/// Minimal URL encoder for socket paths (just encode slashes and special chars).
fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.bytes() {
        match c {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'_' | b'~' | b':' => {
                result.push(c as char);
            }
            b'/' => result.push_str("%2F"),
            _ => {
                result.push_str(&format!("%{:02X}", c));
            }
        }
    }
    result
}

/// Convert a JDBC MySQL URI to a mysql_async compatible connection string.
/// Example: `jdbc:mysql://host:3306/database` → `mysql://host:3306/database`
fn jdbc_uri_to_mysql(uri: &str) -> String {
    // Strip the `jdbc:` prefix
    let without_prefix = uri.strip_prefix("jdbc:").unwrap_or(uri);
    without_prefix.to_string()
}

/// Mask the password in a connection string for safe logging.
fn host_masked(conn_str: &str) -> String {
    if let Some(at_pos) = conn_str.find("@") {
        if let Some(scheme_end) = conn_str.find("://") {
            let scheme = &conn_str[..scheme_end + 3];
            let after_scheme = &conn_str[scheme_end + 3..at_pos];
            let host_part = &conn_str[at_pos..];
            return format!("{}{}:****@{}", scheme, after_scheme.split('@').next().unwrap_or(""), host_part);
        }
    }
    "mysql://***:****@***".to_string()
}

/// Create the `counter` table if it doesn't exist.
/// The table holds a single row with an auto-incrementing count value.
pub async fn init_schema(pool: &Pool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = pool.get_conn().await?;
    let db_name = MySqlConfig::from_env().database.unwrap_or_else(|| "sages".to_string());
    let create_db = format!("CREATE DATABASE IF NOT EXISTS `{}`", db_name);
    let use_db = format!("USE `{}`", db_name);
    create_db.ignore(&mut conn).await?;
    use_db.ignore(&mut conn).await?;
    println!("Database '{}' ready.", db_name);
    r#"
        CREATE TABLE IF NOT EXISTS counter (
            id INT PRIMARY KEY DEFAULT 1,
            count BIGINT NOT NULL DEFAULT 0
        )
    "#.ignore(&mut conn).await?;

    // Seed the single row if it doesn't exist
    r#"
        INSERT INTO counter (id, count) VALUES (1, 0)
        ON DUPLICATE KEY UPDATE id = id
    "#.ignore(&mut conn).await?;

    println!("Database schema initialized.");
    Ok(())
}

/// Increment the counter and return the new value.
pub async fn increment(pool: &Pool) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = pool.get_conn().await?;
    // Atomic increment via SQL
    r#"UPDATE counter SET count = count + 1 WHERE id = 1"#.ignore(&mut conn).await?;
    let (new_count,) = r#"SELECT count FROM counter WHERE id = 1"#.first::<(i64,), _>(&mut conn).await?.unwrap();
    Ok(new_count)
}
