use std::net::SocketAddr;

use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

mod serve;
mod err;
mod support;

const ADDRESS: [u8; 4] = [0, 0, 0, 0];
const PORT: u16 = support::parse_u16(env!("WEB_PORT"));

// hosting platform only has 1 thread
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from((ADDRESS, PORT));
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {addr}");

    loop {
        let builder = http1::Builder::new();
        let tcp_stream = listener.accept();
        let io = TokioIo::new(tcp_stream.await?.0);
        let connection = builder.serve_connection(io, service_fn(serve::serve));

        // we aren't multithreaded but at least we can handle multiple non-blocking connections
        tokio::task::spawn(async move {
            if let Err(err) = connection.await
            {
                println!("Error serving connection: {:?}", err)
            }
        });
    }
}