use html_proc::html;
use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, header::CONTENT_TYPE, Request, Response};

use crate::err::ServerError;

pub async fn style(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, ServerError> {
    Response::builder()
        .status(200)
        .header(CONTENT_TYPE, "text/css")
        .body(Full::new(Bytes::from_static(html!("style.css"))))
        .map_err(|e| e.into())
}