use tokio::net::TcpStream;

use crate::{Headers, ServerError, StatusCode, Version};

#[derive(Default)]
pub struct Response {
    pub head: Parts,
    pub body: String,
}

#[derive(Default)]
pub struct Parts {
    pub status: StatusCode,
    pub version: Version,
    pub headers: Headers,
}

impl Response {
    pub fn new(body: Option<String>) -> Self {
        if let Some(b) = body {
            let mut r = Self::default();
            r.head
                .headers
                .replace("content-length", b.len().to_string())
                .ok();
            r.body = b;
            r
        } else {
            Self::default()
        }
    }
}

impl Parts {
    fn write(&self, w: &mut TcpStream) -> Result<usize, ServerError> {
        let mut n = 0;
        n += self.version.write(w)?;
        n += self.status.write(w)?;
        n += self.headers.write(w)?;
        Ok(n)
    }
}

impl Response {
    pub async fn write(&self, w: &mut TcpStream) -> Result<usize, ServerError> {
        w.writable().await?;

        let mut n = 0;
        n += self.head.write(w)?;
        n += w.try_write(self.body.as_bytes())?;

        Ok(n)
    }
}

impl StatusCode {
    fn write(&self, w: &TcpStream) -> Result<usize, ServerError> {
        match self {
            &StatusCode::OK => Ok(w.try_write("HTTP/1.1 200 OK\r\n".as_bytes())?),
            &StatusCode::BAD_REQUEST => Ok(w.try_write("HTTP/1.1 400 Bad Request\r\n".as_bytes())?),
            &StatusCode::INTERNAL_SERVER_ERROR => {
                Ok(w.try_write("HTTP/1.1 500 Internal Server Error\r\n".as_bytes())?)
            }
            _ => Err(ServerError::Internal),
        }
    }
}

impl Version {
    fn write(&self, w: &TcpStream) -> Result<usize, ServerError> {
        Ok(w.try_write(self.as_str().as_bytes())?)
    }
}

impl Headers {
    fn write(&self, w: &TcpStream) -> Result<usize, ServerError> {
        let mut n = self.headers.iter().try_fold(0 as usize, |acc, h| {
            Ok::<usize, ServerError>(acc + w.try_write(format!("{}: {}\r\n", h.0, h.1).as_bytes())?)
        })?;

        n += w.try_write("\r\n".as_bytes())?;

        Ok(n)
    }
}

impl Headers {
    pub fn default_headers(content_length: u16) -> Result<Headers, ServerError> {
        let mut h = Headers::new();
        h.set("Content-Length".to_string(), content_length.to_string())?;
        h.set("Connection".to_string(), "closed".to_string())?;
        h.set("Content-Type".to_string(), "text/plain".to_string())?;

        Ok(h)
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let body = match self {
            Self::IOError(err) => err.to_string(),
            Self::Internal => "Internal Server Error".to_string(),
            Self::BadRequest => "Bad Request".to_string(),
            Self::Parsing(err) => err.to_string(),
        };

        let head = Parts {
            headers: Headers::default_headers(body.len() as u16).unwrap_or_default(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
            ..Default::default()
        };

        Response { head, body }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

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
