use preload::preload;
use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, header::{CONTENT_ENCODING, CONTENT_TYPE}, Request, Response};

use crate::err::ServerError;

pub async fn home(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, ServerError> {
    Response::builder()
        .status(200)
        .header(CONTENT_TYPE, "text/html")
        .header(CONTENT_ENCODING, "gzip")
        .body(Full::new(Bytes::from_static(preload!("index.html"))))
        .map_err(|e| e.into())
}