use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct ServerError {
    
}

impl From<hyper::http::Error> for ServerError {
    fn from(value: hyper::http::Error) -> Self {
        ServerError {
            
        }
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Client Error")
    }
}

impl Error for ServerError {}