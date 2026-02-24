use core::fmt;
use std::fs;

use crate::{Headers, ServerError, StatusCode, Version};

#[derive(Default)]
pub struct Response {
    pub head: Parts,
    pub body: String,
}

#[derive(Default)]
pub struct Parts {
    pub version: Version,
    pub status: StatusCode,
    pub headers: Headers,
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Head {:?}", self.head)?;
        writeln!(f, "Body {:?}", self.body)?;

        Ok(())
    }
}
impl fmt::Debug for Parts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "- Version: {:?}", self.version)?;
        writeln!(f, "- Status: {:?}", self.status)?;
        writeln!(f, "Headers:")?;
        for (k, v) in self.headers.0.iter() {
            writeln!(f, "– {}: {}", k, v)?;
        }

        Ok(())
    }
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
            let r = Self::default();

            r
        }
    }

    pub fn content_type(mut self, content_type: &str) -> Result<Self, ServerError> {
        self.head
            .headers
            .replace("content-type", content_type.to_string())?;

        Ok(self)
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
            Self::Internal => fs::read_to_string("500.html").unwrap(),
            Self::BadRequest => fs::read_to_string("400.html").unwrap(),
            Self::Parsing(err) => err.to_string(),
        };

        let head = Parts {
            headers: Headers::default_headers(body.len() as u16).unwrap_or_default(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
            ..Default::default()
        };

        let r = Response { head, body }.content_type("text/html").unwrap();

        r
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
