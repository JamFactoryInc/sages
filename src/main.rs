use std::net::SocketAddr;
use std::sync::Arc;

use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

mod serve;
mod err;
mod support;
mod db;
mod mysql_local;

const ADDRESS: [u8; 4] = [0, 0, 0, 0];

cfg_select! {
    windows => {
        const PORT: u16 = 8080;
    }
    target_arch = "aarch64" => {
        const PORT: u16 = 8080;
    }
    _ => {
        const PORT: u16 = support::parse_u16(env!("WEB_PORT"));
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mysql_pool = init_mysql_pool().await;
    init_schema_safe(&mysql_pool).await;
    let mysql_pool = Arc::new(mysql_pool);

    let addr = SocketAddr::from((ADDRESS, PORT));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on {addr}");

    loop {
        let (io, pool) = accept_connection(&listener, &mysql_pool).await?;
        serve_connection(io, pool);
    }
}

/// Initialize the MySQL pool based on the target architecture.
async fn init_mysql_pool() -> Option<mysql_async::Pool> {
    cfg_select! {
        target_arch = "aarch64" => {
            if has_env_config() {
                db::init_pool().await
            } else {
                init_local_mysql().await
            }
        }
        _ => {
            db::init_pool().await
        }
    }
}

/// Returns true if explicit env-var MySQL config is set (e.g., on Northflank).
fn has_env_config() -> bool {
    db::MySqlConfig::from_env().connection_string().is_some()
}

/// Try to start a local MySQL subprocess, falling back to env config on failure.
async fn init_local_mysql() -> Option<mysql_async::Pool> {
    match mysql_local::start_local_mysql() {
        Ok((socket_path, child)) => {
            keep_child_alive(child);
            let pool = db::init_pool_from_socket(&socket_path);
            wait_for_pool_safe(&pool).await;
            pool
        }
        Err(e) => {
            println!("Local MySQL not available: {}. Falling back to env config.", e);
            db::init_pool().await
        }
    }
}

/// Spawn a task that keeps the MySQL child process alive for the lifetime of the app.
fn keep_child_alive(mut child: std::process::Child) {
    tokio::task::spawn(async move {
        match child.wait() {
            Ok(s) => println!("MySQL subprocess exited: {:?}", s),
            Err(e) => eprintln!("MySQL subprocess error: {}", e),
        }
    });
}

/// Wait for the socket-based pool to be ready, logging any timeout.
async fn wait_for_pool_safe(pool: &Option<mysql_async::Pool>) {
    if let Some(ref pool) = pool {
        if let Err(e) = mysql_local::wait_for_pool(pool).await {
            eprintln!("MySQL pool readiness check: {}", e);
        }
    }
}

/// Initialize the database schema if the pool is available, logging any errors.
async fn init_schema_safe(pool: &Option<mysql_async::Pool>) {
    if let Some(ref pool) = pool {
        if let Err(e) = db::init_schema(pool).await {
            eprintln!("Failed to initialize database schema: {}", e);
        }
    }
}

/// Accept a new TCP connection and clone the pool for the handler.
async fn accept_connection(
    listener: &TcpListener,
    pool: &Arc<Option<mysql_async::Pool>>,
) -> Result<(TokioIo<tokio::net::TcpStream>, Arc<Option<mysql_async::Pool>>), Box<dyn std::error::Error>> {
    let io = TokioIo::new(listener.accept().await?.0);
    Ok((io, pool.clone()))
}

/// Spawn a task to serve a single HTTP connection.
fn serve_connection(io: TokioIo<tokio::net::TcpStream>, pool: Arc<Option<mysql_async::Pool>>) {
    let builder = http1::Builder::new();
    let connection = builder.serve_connection(io, service_fn(move |req| serve::serve(req, pool.clone())));
    tokio::task::spawn(async move {
        if let Err(err) = connection.await {
            println!("Error serving connection: {:?}", err)
        }
    });
}