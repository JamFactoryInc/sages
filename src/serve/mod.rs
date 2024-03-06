use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, Request, Response};

use crate::err::ServerError;

pub mod html;
pub mod err;
pub mod css;

pub async fn serve(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, ServerError> {
    match req.uri().path() {
        "" | "/" => Ok(html::home(req).await?),
        "/style.css" => Ok(css::style(req).await?),
        _ => Ok(err::not_found(req).await?)
    }
}