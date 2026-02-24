use crate::{HTTPParsingError, Request};
use std::collections::HashMap;

impl Request {
    pub fn parse_body(&mut self, b: &[u8]) -> Result<(usize, bool), HTTPParsingError> {
        let (mut read, mut done) = (0, false);

        if let Some(cl) = self.head.headers.get("content-length") {
            let n: usize = cl.parse()?;
            let remaining = n - self.body.len();
            let m = remaining.min(b.len());
            let s = std::str::from_utf8(&b[..m])?;
            self.body += s;

            read += m;

            if self.body.len() == n {
                done = true;
            }
        } else {
            done = true;
        }

        Ok((read, done))
    }
}

/// A frame of any kind related to an HTTP stream (body).
pub struct Frame<T> {
    kind: Kind<T>,
}

enum Kind<T> {
    Data(T),
    Trailers(HashMap<String, String>),
}

impl<T> Frame<T> {
    pub fn data(buf: T) -> Self {
        Self {
            kind: Kind::Data(buf),
        }
    }

    pub fn into_data(self) -> Result<T, Self> {
        match self.kind {
            Kind::Data(data) => Ok(data),
            _ => Err(self),
        }
    }

    pub fn is_data(&self) -> bool {
        matches!(self.kind, Kind::Data(..))
    }

    pub fn data_ref(&self) -> Option<&T> {
        match self.kind {
            Kind::Data(ref data) => Some(data),
            _ => None,
        }
    }
}
