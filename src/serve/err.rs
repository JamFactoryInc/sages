use preload::preload;
use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, Request, Response};

use crate::err::ServerError;

use super::ContentType;

pub async fn not_found(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, ServerError> {
    super::not_found(ContentType::Html)
        .body(super::bytes(preload!("404.html")))
        .map_err(|e| e.into())
}

pub async fn method_not_allowed(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, ServerError> {
    Response::builder()
        .status(405)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(r#"{"error": "method not allowed"}"#)))
        .map_err(|e| e.into())
}