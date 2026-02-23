mod encoder;
mod error;
mod parser;
mod parts;
mod pool;
mod request;
mod response;
mod server;

pub use encoder::*;
pub use error::*;
pub use parser::*;
pub use pool::*;
pub const SERVER_PORT: u16 = 42069;

pub use parts::*;
pub use request::Request;
pub use response::{IntoResponse, Response};
pub use server::*;

const SEPARATOR: &[u8] = b"\r\n";
