use std::io::{self, Read, read_to_string};
use thiserror::Error;

pub struct Request {
    pub request_line: RequestLine,
    pub state: ParserState,
}

pub enum ParserState {
    Init,
    Done,
}

pub struct RequestLine {
    pub http_version: String,
    pub request_target: String,
    pub method: String,
}

pub enum HTTPMethod {
    GET,
    POST,
    DELETE,
    PATCH,
    PUT,
}

const SEPARATOR: &str = "\r\n";

#[derive(Error, Debug)]
pub enum HTTPParsingError {
    #[error("malformed request line")]
    BadRequestLine,
    #[error("unsupported http version")]
    UnsupportedHTTPVersion,
    #[error("request line not found")]
    RequestLineNotFound,

    #[error("reader error")]
    IOError(#[from] io::Error),
}

impl Request {
    pub fn new() -> Self {
        Request {
            request_line: (),
            state: ParserState::Init,
        }
    }

    pub fn from_reader(r: impl Read) -> Result<Self, HTTPParsingError> {
        let s = read_to_string(r)?;

        let (rl, _rest) = Self::parse_request_line(&s)?;

        Ok(Request { request_line: rl })
    }

    fn parse_request_line(s: &str) -> Result<(RequestLine, u32), HTTPParsingError> {
        if let Some(i) = s.find(SEPARATOR) {
            let start_line = &s[..i];

            // DO NOT INLUCDE THE SEPARATOR IN THE REST
            let rest_of_message = &s[i + SEPARATOR.len()..];

            let mut parts = start_line.split(' ');

            let method = parts
                .next()
                .filter(|m| !m.is_empty() && m.chars().all(|c| c.is_ascii_uppercase()))
                .map(str::to_string)
                .ok_or(HTTPParsingError::BadRequestLine)?;
            let request_target = parts
                .next()
                .ok_or(HTTPParsingError::BadRequestLine)?
                .to_string();
            let http_version = parts
                .next()
                .and_then(|s| s.split_once("/"))
                .filter(|(proto, v)| *proto == "HTTP" && *v == "1.1")
                .map(|(_, v)| v.to_string())
                .ok_or(HTTPParsingError::BadRequestLine)?;

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
            Ok((rl, rest_of_message))
        } else {
            Err(HTTPParsingError::RequestLineNotFound)
        }
    }
}

impl RequestLine {
    fn valid_http(&self) -> bool {
        self.http_version == "1.1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Read};

    struct ChunkReader {
        data: String,
        num_bytes_per_read: usize,
        pos: usize,
    }

    impl ChunkReader {
        fn new(data: &str, num_bytes_per_read: usize) -> Self {
            Self {
                data: data.to_string(),
                num_bytes_per_read,
                pos: 0,
            }
        }
    }

    impl Read for ChunkReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.pos >= self.data.len() {
                return Ok(0); // EOF
            }

            let end_index = (self.pos + self.num_bytes_per_read).min(self.data.len());

            let bytes = self.data.as_bytes();
            let n = buf.len().min(end_index - self.pos);

            buf[..n].copy_from_slice(&bytes[self.pos..self.pos + n]);
            self.pos += n;

            Ok(n)
        }
    }

    #[test]
    fn good_get_request_line() {
        let r = Request::from_reader(
            ChunkReader::new("GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n", 3)
        ).unwrap();
        assert_eq!("GET", r.request_line.method);
        assert_eq!("/", r.request_line.request_target);
        assert_eq!("1.1", r.request_line.http_version);
    }

    #[test]
    fn good_get_request_line_with_path() {
        let r = Request::from_reader(
            ChunkReader::new("GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n", 1),
        ).unwrap();
        assert_eq!("GET", r.request_line.method);
        assert_eq!("/coffee", r.request_line.request_target);
        assert_eq!("1.1", r.request_line.http_version);
    }

    #[test]
    fn bad_input() {
        let r = Request::from_reader(ChunkReader::new(
            "/coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n",
            2,
        ));
        assert!(r.is_err());
    }
}
