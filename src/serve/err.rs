use preload::preload;
use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, header::CONTENT_TYPE, Request, Response};

use crate::err::ServerError;

pub async fn not_found(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, ServerError> {
    Response::builder()
        .status(404)
        .header(CONTENT_TYPE, "text/html")
        .body(Full::new(Bytes::from_static(preload!("404.html"))))
        .map_err(|e| e.into())
}