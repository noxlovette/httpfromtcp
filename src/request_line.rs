use crate::{HTTPParsingError, Request, SEPARATOR};
use std::fmt;
pub struct RequestLine {
    pub http_version: String,
    pub request_target: String,
    pub method: String,
}

impl fmt::Display for RequestLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Request line:")?;
        writeln!(f, "- Method: {}", self.method)?;
        writeln!(f, "- Target: {}", self.request_target)?;
        writeln!(f, "- Version: {}", self.http_version)
    }
}

impl RequestLine {
    fn valid_http(&self) -> bool {
        self.http_version == "1.1"
    }
}
impl Request {
    pub fn parse_request_line(b: &[u8]) -> Result<(Option<RequestLine>, usize), HTTPParsingError> {
        if let Some(i) = b.windows(SEPARATOR.len()).position(|w| w == SEPARATOR) {
            let start_line = &b[..i];

            let read = i + SEPARATOR.len();

            let mut parts = start_line.split(|&b| b == b' ');

            let method = parts
                .next()
                .filter(|tok| !tok.is_empty() && tok.iter().all(|&c| c.is_ascii_uppercase()))
                .and_then(|tok| std::str::from_utf8(tok).ok())
                .ok_or(HTTPParsingError::BadRequestLine)?
                .to_string();

            let request_target = parts
                .next()
                .filter(|t| !t.is_empty()) // add your target rules here
                .and_then(|tok| std::str::from_utf8(tok).ok())
                .ok_or(HTTPParsingError::BadRequestLine)?
                .to_string();

            let http_version = parts
                .next()
                .ok_or(HTTPParsingError::BadRequestLine)
                .and_then(|tok| {
                    std::str::from_utf8(tok).map_err(|_| HTTPParsingError::BadRequestLine)
                })
                .and_then(|s| {
                    let (proto, v) = s.split_once('/').ok_or(HTTPParsingError::BadRequestLine)?;
                    if proto != "HTTP" || !(v == "1.1" || v == "1.0") {
                        return Err(HTTPParsingError::BadRequestLine);
                    }
                    Ok(v.to_string())
                })?;

            if parts.next().is_some() {
                return Err(HTTPParsingError::BadRequestLine);
            }

            let rl = RequestLine {
                method,
                request_target,
                http_version,
            };

            if !rl.valid_http() {
                return Err(HTTPParsingError::UnsupportedHTTPVersion);
            }
            Ok((Some(rl), read))
        } else {
            Ok((None, 0))
        }
    }
}
