use std::env;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;

use mysql_async::Pool;

/// Path to the homebrew MySQL installation on aarch64 macOS.
const MYSQL_PREFIX: &str = "/opt/homebrew/opt/mysql";

/// Start a local MySQL instance as a subprocess.
/// Returns the socket path and a handle to the child process.
/// The caller is responsible for keeping the child alive while using the connection.
pub fn start_local_mysql() -> Result<(String, Child), String> {
    let mysql_home = PathBuf::from(MYSQL_PREFIX);
    let mysqld = mysql_home.join("bin/mysqld");

    if !mysqld.exists() {
        return Err(format!("mysqld not found at {}", mysqld.display()));
    }

    // Use a unique data directory under the project's target folder (absolute path)
    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().expect("current dir").join("target"));
    let data_dir = target_dir.join("mysql_local_data");

    // Create data dir if it doesn't exist
    std::fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data dir: {}", e))?;

    // Check if data directory is already initialized (contains ibdata1)
    let needs_init = !data_dir.join("ibdata1").exists();

    let socket_path = data_dir.join("mysql.sock");
    let socket_str = socket_path.to_string_lossy().to_string();

    // Initialize data directory if needed
    if needs_init {
        let mysql_install_db = mysql_home.join("bin/mysql_install_db");
        if !mysql_install_db.exists() {
            // Fallback: mysqld --initialize-insecure
            println!("Initializing MySQL data directory...");
            let init_status = Command::new(&mysqld)
                .args(&["--initialize-insecure", "--datadir", data_dir.to_str().unwrap()])
                .status()
                .map_err(|e| format!("Failed to run mysqld --initialize-insecure: {}", e))?;

            if !init_status.success() {
                return Err("mysqld --initialize-insecure failed".to_string());
            }
        } else {
            println!("Initializing MySQL data directory...");
            let init_status = Command::new(&mysql_install_db)
                .args(&["--datadir", data_dir.to_str().unwrap()])
                .status()
                .map_err(|e| format!("Failed to run mysql_install_db: {}", e))?;

            if !init_status.success() {
                return Err("mysql_install_db failed".to_string());
            }
        }
    }

    // Start mysqld
    println!("Starting local MySQL on socket: {}", socket_str);
    let child = Command::new(&mysqld)
        .args(&[
            "--datadir", data_dir.to_str().unwrap(),
            "--socket", &socket_str,
            "--port", "0", // disable TCP, use socket only
            "--skip-grant-tables",
            "--skip-networking",
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start mysqld: {}", e))?;

    // Wait for the socket to appear
    for _ in 0..60 {
        // 60 * 100ms = 6 seconds max wait
        std::thread::sleep(Duration::from_millis(100));
        if socket_path.exists() {
            break;
        }
    }

    if !socket_path.exists() {
        return Err("Timed out waiting for MySQL socket".to_string());
    }

    Ok((socket_str, child))
}

/// Wait for a MySQL pool to become ready (socket-based local instance may need a moment).
pub async fn wait_for_pool(pool: &Pool) -> Result<(), String> {
    for _ in 0..30 {
        // 30 * 200ms = 6 seconds max wait
        match pool.get_conn().await {
            Ok(_) => return Ok(()),
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
    Err("Timed out waiting for MySQL to be ready".to_string())
}
