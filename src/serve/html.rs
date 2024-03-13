use preload::preload;
use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, Request, Response};

use crate::err::ServerError;

use super::ContentType;

pub async fn home(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, ServerError> {
    super::ok(ContentType::Html)
        .body(super::bytes(preload!("index.html")))
        .map_err(|e| e.into())
}