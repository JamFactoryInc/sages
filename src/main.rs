use std::{convert::Infallible, net::SocketAddr};

use err::ClientError;
use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

mod err;
mod html;

// hosting platform doesn't have many resources, so we need to be lightweight
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let file = include_bytes!("err.rs");
    println!("{file:?}");

    // Render expects binding to 0.0.0.0:10000
    let addr = SocketAddr::from(([0, 0, 0, 0], 10000));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // we aren't multithreaded but at least we can handle multiple non-blocking connections
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(serve))
                .await
            {
                println!("Error serving connection: {:?}", err)
            }
        });
    }
}

async fn serve(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, ClientError> {
    match req.uri().path() {
        "" | "/" => Ok(home(req).await?),
        _ => Err(ClientError {})
    }
}

async fn home(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, ClientError> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}