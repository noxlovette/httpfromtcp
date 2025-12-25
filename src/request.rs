use std::io::{self, Read, read_to_string};
use thiserror::Error;

#[derive(Default)]
pub struct Request {
    pub request_line: Option<RequestLine>,
    pub state: ParserState,
}

#[derive(PartialEq, Default)]
pub enum ParserState {
    #[default]
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

const SEPARATOR: &[u8] = b"\r\n";

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
        Self::default()
    }

    pub fn done(&self) -> bool {
        self.state == ParserState::Done
    }

    pub fn from_reader(mut r: impl Read) -> Result<Self, HTTPParsingError> {
        let mut req = Request::new();

        let mut buf = [0u8; 1024];
        let mut buf_len = 0;
        while !req.done() {
            let n = r.read(&mut buf[buf_len..])?;
            if n == 0 {
                break;
            }
            buf_len += n;

            let read = req.parse(&buf[..buf_len])?;
            buf.copy_within(read..buf_len, 0);
            buf_len -= read;
        }

        Ok(req)
    }

    fn parse(&mut self, data: &[u8]) -> Result<usize, HTTPParsingError> {
        let mut read: usize = 0;
        loop {
            match self.state {
                ParserState::Init => {
                    let (rl, n) = Self::parse_request_line(&data[read..])?;

                    if n == 0 {
                        break;
                    }
                    self.request_line = rl;
                    read += n;

                    self.state = ParserState::Done;
                }
                ParserState::Done => break,
            }
        }

        return Ok(read);
    }

    fn parse_request_line(b: &[u8]) -> Result<(Option<RequestLine>, usize), HTTPParsingError> {
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

        let rl = r.request_line.unwrap();
        assert_eq!("GET", rl.method);
        assert_eq!("/", rl.request_target);
        assert_eq!("1.1", rl.http_version);
    }

    #[test]
    fn good_get_request_line_with_path() {
        let r = Request::from_reader(
            ChunkReader::new("GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n", 1),
        ).unwrap();

        let rl = r.request_line.unwrap();

        assert_eq!("GET", rl.method);
        assert_eq!("/coffee", rl.request_target);
        assert_eq!("1.1", rl.http_version);
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
