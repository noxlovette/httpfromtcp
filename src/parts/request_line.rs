use crate::{HTTPParsingError, Request, SEPARATOR, Version, parts::method::Method};

use std::fmt;
pub struct RequestLine {
    pub version: Version,
    pub uri: String,
    pub method: Method,
}

impl fmt::Debug for RequestLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Request line:")?;
        writeln!(f, "- Method: {:?}", self.method)?;
        writeln!(f, "- Target: {:?}", self.uri)?;
        writeln!(f, "- Version: {:?}", self.version)
    }
}

impl Request {
    pub fn parse_request_line(b: &[u8]) -> Result<(Option<RequestLine>, usize), HTTPParsingError> {
        if let Some(i) = b.windows(SEPARATOR.len()).position(|w| w == SEPARATOR) {
            let start_line = &b[..i];

            let read = i + SEPARATOR.len();

            let mut parts = start_line.split(|&b| b == b' ');

            let method = Method::from_bytes(
                parts
                    .next()
                    .filter(|tok| !tok.is_empty() && tok.iter().all(|&c| c.is_ascii_uppercase()))
                    .ok_or_else(|| HTTPParsingError::BadRequestLine)?,
            )?;

            let uri = parts
                .next()
                .filter(|t| !t.is_empty())
                .and_then(|tok| std::str::from_utf8(tok).ok())
                .ok_or(HTTPParsingError::BadRequestLine)?
                .to_string();

            let version =
                Version::from_bytes(parts.next().ok_or(HTTPParsingError::BadRequestLine)?)?;

            if parts.next().is_some() {
                return Err(HTTPParsingError::BadRequestLine);
            }

            let rl = RequestLine {
                method,
                uri,
                version,
            };

            Ok((Some(rl), read))
        } else {
            Ok((None, 0))
        }
    }
}
