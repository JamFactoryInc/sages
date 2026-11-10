use std::sync::Arc;

use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, header::{CONTENT_ENCODING, CONTENT_TYPE}, Method, Request, Response};

use crate::db;
use crate::err::ServerError;
use mysql_async::Pool;

pub mod html;
pub mod err;
pub mod css;
pub mod js;

pub async fn serve(req: Request<Incoming>, pool: Arc<Option<Pool>>) -> Result<Response<Full<Bytes>>, ServerError> {
    match req.uri().path() {
        "" | "/" => Ok(html::home(req).await?),
        "/style.css" => Ok(css::style(req).await?),
        "/index.js" => Ok(js::index(req).await?),
        "/favicon.svg" => Ok(css::favicon(req).await?),
        "/inc" => {
            if *req.method() == Method::POST {
                return Ok(handle_inc(pool).await);
            }
            Ok(err::method_not_allowed(req).await?)
        }
        _ => Ok(err::not_found(req).await?)
    }
}

async fn handle_inc(pool: Arc<Option<Pool>>) -> Response<Full<Bytes>> {
    match pool.as_ref() {
        Some(pool) => {
            match db::increment(pool).await {
                Ok(count) => {
                    let body = format!("{{\"count\": {}}}", count);
                    Response::builder()
                        .status(200)
                        .header(CONTENT_TYPE, "application/json")
                        .body(Full::new(Bytes::from(body)))
                        .unwrap()
                }
                Err(e) => {
                    eprintln!("Failed to increment counter: {}", e);
                    let body = format!("{{\"error\": \"database error\"}}");
                    Response::builder()
                        .status(500)
                        .header(CONTENT_TYPE, "application/json")
                        .body(Full::new(Bytes::from(body)))
                        .unwrap()
                }
            }
        }
        None => {
            let body = "{\"error\": \"database not configured\"}";
            Response::builder()
                .status(503)
                .header(CONTENT_TYPE, "application/json")
                .body(Full::new(Bytes::from(body)))
                .unwrap()
        }
    }
}

#[derive(Clone, Copy)]
pub enum ContentType {
    Html,
    Css,
    Js,
    Svg,
}

pub fn get_response(content_type: ContentType) -> hyper::http::response::Builder {
    let content_type_val = match content_type {
        ContentType::Html => "text/html",
        ContentType::Css => "text/css",
        ContentType::Js => "text/javascript",
        ContentType::Svg => "image/svg+xml",
    };
    Response::builder()
        .status(200)
        .header(CONTENT_TYPE, content_type_val)
        .header(CONTENT_ENCODING, "gzip")
}

pub fn bytes(bytes: &'static [u8]) -> Full<Bytes> {
    Full::new(Bytes::from_static(bytes))
}

pub fn ok(content_type: ContentType) -> hyper::http::response::Builder {
    get_response(content_type).status(200)
}

pub fn not_found(content_type: ContentType) -> hyper::http::response::Builder {
    get_response(content_type).status(404)
}