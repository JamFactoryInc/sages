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
    /// - `MYSQL_JDBC_URI`: Full JDBC connection string (e.g., `jdbc:mysql://host:port/database`)
    /// - `HOST`, `PORT`, `USERNAME`, `PASSWORD`, `DATABASE`: Individual connection details
    pub fn from_env() -> Self {
        Self {
            uri: env::var("MYSQL_JDBC_URI").ok(),
            host: env::var("HOST").ok(),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok()),
            username: env::var("USERNAME").ok(),
            password: env::var("PASSWORD").ok(),
            database: env::var("DATABASE").ok(),
        }
    }

    /// Build a connection string from the configuration.
    /// Prefers `MYSQL_JDBC_URI` if available, converting it from JDBC format; otherwise constructs from individual parts.
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
    let config = MySqlConfig::from_env();
    
    match config.connection_string() {
        Some(conn_str) => {
            init_pool_from_conn_str(&conn_str).await
        }
        None => {
            println!("No MySQL configuration found. Skipping database connection.");
            None
        }
    }
}

/// Create a MySQL connection pool from a specific connection string.
pub async fn init_pool_from_conn_str(conn_str: &str) -> Option<Pool> {
    println!("Connecting to MySQL: {}", host_masked(conn_str));

    // Parse the connection string into Opts
    let opts = match conn_str.parse::<mysql_async::Opts>() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Invalid MySQL connection string: {}", e);
            return None;
        }
    };

    let pool = Pool::new(opts);

    // Test the connection
    match pool.get_conn().await {
        Ok(_conn) => {
            println!("MySQL connection established successfully.");
            Some(pool)
        }
        Err(e) => {
            eprintln!("Failed to connect to MySQL: {}", e);
            None
        }
    }
}

/// Create a MySQL connection pool connected via Unix socket.
/// Skips the initial test connection to avoid blocking the single-threaded runtime;
/// the caller should use `mysql_local::wait_for_pool` to verify readiness.
pub fn init_pool_from_socket(socket_path: &str) -> Option<Pool> {
    println!("Connecting to MySQL via socket: {}", socket_path);

    let encoded_socket = urlencoding(socket_path);
    let conn_str = format!("mysql://root@localhost/?socket={}", encoded_socket);

    let opts: mysql_async::Opts = match conn_str.parse() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Invalid MySQL socket connection string: {}", e);
            return None;
        }
    };

    Some(Pool::new(opts))
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
    r#"CREATE DATABASE IF NOT EXISTS sages"#.ignore(&mut conn).await?;
    r#"USE sages"#.ignore(&mut conn).await?;
    println!("Database 'sages' ready.");
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
