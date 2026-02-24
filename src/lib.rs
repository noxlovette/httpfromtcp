mod connection;
mod encoder;
mod error;
mod listener;
mod parts;
mod request;
mod response;
mod server;

pub use connection::*;
pub use encoder::Encode;
pub use error::*;
pub use listener::*;
pub const SERVER_PORT: u16 = 42069;
pub use parts::*;
pub use request::Request;
pub use response::{IntoResponse, Response};
pub use server::*;

const SEPARATOR: &[u8] = b"\r\n";
