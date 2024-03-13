use preload::preload;
use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, Request, Response};

use crate::err::ServerError;

use super::ContentType;

pub async fn style(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, ServerError> {
    super::ok(ContentType::Css)
        .body(super::bytes(preload!("style.css")))
        .map_err(|e| e.into())
}

pub async fn favicon(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, ServerError> {
    super::ok(ContentType::Svg)
        .body(super::bytes(preload!("favicon.svg")))
        .map_err(|e| e.into())
}