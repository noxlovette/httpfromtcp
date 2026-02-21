use std::io::Write;

use crate::{Headers, ServerError, StatusCode};

pub struct Response;

impl StatusCode {
    pub fn write(&self, w: &mut impl Write) -> Result<usize, ServerError> {
        match self {
            &StatusCode::OK => Ok(w.write("HTTP/1.1 200 OK\r\n".as_bytes())?),
            &StatusCode::BAD_REQUEST => Ok(w.write("HTTP/1.1 400 Bad Request\r\n".as_bytes())?),
            &StatusCode::INTERNAL_SERVER_ERROR => {
                Ok(w.write("HTTP/1.1 500 Internal Server Error\r\n".as_bytes())?)
            }
            _ => Err(ServerError::Internal),
        }
    }
}

impl Headers {
    pub fn write(&self, w: &mut impl Write) -> Result<usize, ServerError> {
        let mut n = self.headers.iter().try_fold(0 as usize, |acc, h| {
            Ok::<usize, ServerError>(acc + w.write(format!("{}: {}\r\n", h.0, h.1).as_bytes())?)
        })?;

        n += w.write("\r\n".as_bytes())?;

        Ok(n)
    }

    pub fn default_headers(content_length: u16) -> Result<Headers, ServerError> {
        let mut h = Headers::new();
        h.set("Content-Length".to_string(), content_length.to_string())?;
        h.set("Connection".to_string(), "closed".to_string())?;
        h.set("Content-Type".to_string(), "text/plain".to_string())?;

        Ok(h)
    }
}
