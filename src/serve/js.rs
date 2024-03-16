use preload::preload;
use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, Request, Response};

use crate::err::ServerError;

use super::ContentType;

pub async fn index(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, ServerError> {
    super::ok(ContentType::Js)
        .body(super::bytes(preload!("dist/main.js")))
        .map_err(|e| e.into())
}