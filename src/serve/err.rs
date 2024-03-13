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