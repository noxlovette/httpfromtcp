mod error;
mod parser;
mod parts;
mod request;
mod response;
mod server;

pub use error::*;
pub use parser::*;
pub const SERVER_PORT: u16 = 42069;

pub use parts::*;
pub use request::*;
pub use response::*;
pub use server::*;

const SEPARATOR: &[u8] = b"\r\n";

pub trait IntoResponse {
    #[must_use]
    fn into_response(self) -> Response;
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(value) => value.into_response(),
            Err(err) => err.into_response(),
        }
    }
}
