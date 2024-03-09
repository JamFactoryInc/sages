use std::{convert::Infallible, fmt::format, future::Future, net::{SocketAddr, TcpStream}};

use err::ServerError;
use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, header::{HeaderName, HeaderValue, CONTENT_TYPE}, server::conn::http1::{self, Connection}, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

mod serve;
mod err;

// Render expects binding to 0.0.0.0:10000
const ADDRESS: [u8; 4] = [0, 0, 0, 0];
const PORT: u16 = 10000;

// hosting platform only has 1 thread
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
 
    let addr = SocketAddr::from((ADDRESS, PORT));
    let listener = TcpListener::bind(addr).await?;

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