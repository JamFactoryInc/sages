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
    // On aarch64, try to start a local MySQL subprocess; otherwise use env-var config
    let mysql_pool = cfg_select! {
        target_arch = "aarch64" => {
            // Prefer explicit env-var config if set (e.g., Northflank)
            if db::MySqlConfig::from_env().connection_string().is_some() {
                db::init_pool().await
            } else {
                // Try to start local MySQL via homebrew
                match mysql_local::start_local_mysql() {
                    Ok((socket_path, mut child)) => {
                        // Keep the child process alive for the lifetime of the app
                        tokio::task::spawn(async move {
                            let status = child.wait();
                            match status {
                                Ok(s) => println!("MySQL subprocess exited: {:?}", s),
                                Err(e) => eprintln!("MySQL subprocess error: {}", e),
                            }
                        });

                        let pool = db::init_pool_from_socket(&socket_path);
                        // Wait for the socket-based pool to be ready
                        if let Some(ref pool) = pool {
                            if let Err(e) = mysql_local::wait_for_pool(pool).await {
                                eprintln!("MySQL pool readiness check: {}", e);
                            }
                        }
                        pool
                    }
                    Err(e) => {
                        println!("Local MySQL not available: {}. Falling back to env config.", e);
                        db::init_pool().await
                    }
                }
            }
        }
        _ => {
            db::init_pool().await
        }
    };

    // Initialize database schema if pool is available
    if let Some(ref pool) = mysql_pool {
        if let Err(e) = db::init_schema(pool).await {
            eprintln!("Failed to initialize database schema: {}", e);
        }
    }

    let mysql_pool = Arc::new(mysql_pool);

    let addr = SocketAddr::from((ADDRESS, PORT));
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {addr}");

    loop {
        let builder = http1::Builder::new();
        let tcp_stream = listener.accept();
        let io = TokioIo::new(tcp_stream.await?.0);
        let pool = mysql_pool.clone();
        let connection = builder.serve_connection(io, service_fn(move |req| serve::serve(req, pool.clone())));

        tokio::task::spawn(async move {
            if let Err(err) = connection.await
            {
                println!("Error serving connection: {:?}", err)
            }
        });
    }
}