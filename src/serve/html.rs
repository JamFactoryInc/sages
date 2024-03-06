use html_proc::html;
use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, header::CONTENT_TYPE, Request, Response};

use crate::err::ServerError;

pub async fn home(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, ServerError> {
    Response::builder()
        .status(200)
        .header(CONTENT_TYPE, "text/html")
        .body(Full::new(Bytes::from_static(html!("index.html"))))
        .map_err(|e| e.into())
}