mod body;
mod headers;
mod request;
mod request_line;
mod server;
pub const SERVER_PORT: u16 = 42069;
use std::{fmt, io, num::ParseIntError, str::Utf8Error};

pub use headers::*;
pub use request::*;
pub use request_line::*;
pub use server::*;
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
    #[error("invalid body")]
    BadBody,

    #[error("parser error")]
    Parser,

    #[error("reader error")]
    IOError(#[from] io::Error),

    #[error("formatting error")]
    FmtError(#[from] fmt::Error),

    #[error("int parsing error")]
    IntError(#[from] ParseIntError),

    #[error("uft8 parsing error")]
    UtfError(#[from] Utf8Error),
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("IO error")]
    IOError(#[from] io::Error),
}
