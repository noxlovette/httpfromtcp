mod headers;
mod request;
use std::{fmt, io};

pub use headers::*;
pub use request::*;
use thiserror::Error;

pub enum HTTPMethod {
    GET,
    POST,
    DELETE,
    PATCH,
    PUT,
}

const SEPARATOR: &[u8] = b"\r\n";

#[derive(Error, Debug)]
pub enum HTTPParsingError {
    #[error("malformed request line")]
    BadRequestLine,
    #[error("unsupported http version")]
    UnsupportedHTTPVersion,
    #[error("request line not found")]
    RequestLineNotFound,
    #[error("malformed field line")]
    BadFieldLine,
    #[error("invalid token")]
    BadToken,

    #[error("reader error")]
    IOError(#[from] io::Error),

    #[error("formatting error")]
    FmtError(#[from] fmt::Error),
}
