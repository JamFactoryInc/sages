use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct ClientError {
    
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Client Error")
    }
}

impl Error for ClientError {}