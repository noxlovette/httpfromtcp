use std::{fmt, io, num::ParseIntError, str::Utf8Error};
use thiserror::Error;

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
    #[error("bad status code")]
    BadStatusCode,
    #[error("bad method")]
    BadMethod,

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
    #[error("internal error")]
    Internal,

    #[error("bad request")]
    BadRequest,

    #[error("IO error")]
    IOError(#[from] io::Error),

    #[error("Parsing error")]
    Parsing(#[from] HTTPParsingError),
}
