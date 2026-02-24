use crate::response::Parts;
use crate::{Headers, Response, ServerError, StatusCode, Version};
use std::io::Write;
use std::net::TcpStream;

pub trait Encode {
    fn write(&self, w: &mut TcpStream) -> Result<(), ServerError>;
}

impl Encode for Parts {
    fn write(&self, w: &mut TcpStream) -> Result<(), ServerError> {
        self.status.write(w)?;
        self.headers.write(w)?;

        Ok(())
    }
}

impl Encode for Response {
    fn write(&self, w: &mut TcpStream) -> Result<(), ServerError> {
        self.head.write(w)?;
        w.write_all(self.body.as_bytes())?;

        Ok(())
    }
}

impl Encode for StatusCode {
    fn write(&self, w: &mut TcpStream) -> Result<(), ServerError> {
        match self {
            &StatusCode::OK => Ok(w.write_all("HTTP/1.1 200 OK\r\n".as_bytes())?),
            &StatusCode::BAD_REQUEST => Ok(w.write_all("HTTP/1.1 400 Bad Request\r\n".as_bytes())?),
            &StatusCode::INTERNAL_SERVER_ERROR => {
                Ok(w.write_all("HTTP/1.1 500 Internal Server Error\r\n".as_bytes())?)
            }
            _ => Err(ServerError::Internal),
        }
    }
}

impl Encode for Version {
    fn write(&self, w: &mut TcpStream) -> Result<(), ServerError> {
        Ok(w.write_all(self.as_str().as_bytes())?)
    }
}

impl Encode for Headers {
    fn write(&self, w: &mut TcpStream) -> Result<(), ServerError> {
        self.0
            .iter()
            .try_for_each(|h| w.write_all(format!("{}: {}\r\n", h.0, h.1).as_bytes()))?;

        w.write_all("\r\n".as_bytes())?;

        Ok(())
    }
}
