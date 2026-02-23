use crate::{HTTPParsingError, Headers, Method, ParserState, Version};
use std::{fmt, io::Read, task::Context};
use tokio::{io::AsyncRead, net::TcpStream};

#[derive(Default)]
pub struct Request {
    pub head: Parts,
    pub body: String,
    pub state: ParserState,
}

#[derive(Default)]
pub struct Parts {
    pub method: Method,
    pub uri: String,
    pub version: Version,
    pub headers: Headers,
}

impl fmt::Debug for Parts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "- Method: {:?}", self.method)?;
        writeln!(f, "- Target: {:?}", self.uri)?;
        writeln!(f, "- Version: {:?}", self.version)?;
        writeln!(f, "Headers:")?;
        for (k, v) in self.headers.headers.iter() {
            writeln!(f, "– {}: {}", k, v)?;
        }

        Ok(())
    }
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

    pub fn from_reader_async(r: &TcpStream) -> Result<Self, HTTPParsingError> {
        let mut req = Request::new();

        let mut buf = [0u8; 1024];
        let mut buf_len = 0;
        while !req.done() {
            let n = r.try_read(&mut buf[buf_len..])?;
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
            let current_data = &data[read..];

            match self.state {
                ParserState::Init => {
                    let (rl, n) = Self::parse_request_line(current_data)?;

                    if n == 0 {
                        break;
                    }

                    let request_line = rl.ok_or_else(|| HTTPParsingError::BadRequestLine)?;

                    self.head.method = request_line.method;
                    self.head.version = request_line.version;
                    self.head.uri = request_line.uri;

                    read += n;

                    self.state = ParserState::Headers;
                }
                ParserState::Headers => {
                    let (n, done) = self.head.headers.parse(current_data)?;

                    if n == 0 {
                        break;
                    }
                    read += n;
                    if done {
                        self.state = ParserState::Body;
                    }
                }
                ParserState::Body => {
                    let (n, done) = self.parse_body(current_data)?;

                    if n == 0 || current_data.len() == 0 {
                        break;
                    }

                    read += n;

                    if done {
                        self.state = ParserState::Done;
                    }
                }
                ParserState::Done => break,

                ParserState::Error => return Err(HTTPParsingError::Parser),
            }
        }
        return Ok(read);
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

        assert_eq!("GET", r.head.method.as_str());
        assert_eq!("/", r.head.uri);
        assert_eq!("1.1", r.head.version.as_str());
    }

    #[test]
    fn good_get_request_line_with_path() {
        let r = Request::from_reader(
            ChunkReader::new("GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n", 1),
        ).unwrap();

        assert_eq!("GET", r.head.method.as_str());
        assert_eq!("/coffee", r.head.uri);
        assert_eq!("1.1", r.head.version.as_str());
    }

    #[test]
    fn good_parse_headers() {
        let r = Request::from_reader(
            ChunkReader::new("GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n ", 3),
        ).unwrap();

        let h = r.head.headers;
        assert_eq!("localhost:42069", h.get("host").unwrap());
        assert_eq!("curl/7.81.0", h.get("user-agent").unwrap());
        assert_eq!("*/*", h.get("accept").unwrap());
    }

    #[test]
    fn good_parse_body() {
        let r = Request::from_reader(ChunkReader::new(
            "POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nContent-Length: 13\r\n\r\nhello world!\n",
            3,
        ))
        .unwrap();

        assert_eq!("hello world!\n", r.body);
    }
    #[test]
    fn good_parse_empty_body_no_cl_no_body() {
        let r = Request::from_reader(ChunkReader::new(
            "POST /submit HTTP/1.1\r\nHost: localhost:42069\r\n\r\n
		",
            3,
        ));

        assert!(r.is_ok())
    }
    #[test]
    fn good_parse_empty_body_no_cl_empty_body() {
        let r = Request::from_reader(ChunkReader::new(
            "POST /submit HTTP/1.1\r\nHost: localhost:42069\r\n\r\n
		",
            3,
        ));

        assert!(r.is_ok())
    }

    #[test]
    fn bad_parse_body_shorter_content_length() {
        let r = Request::from_reader(ChunkReader::new(
            "POST /submit HTTP/1.1\r\n
		Host: localhost:42069\r\n
		Content-Length: 20\r\n
		\r\n
		hello world!\n",
            3,
        ));

        assert!(r.is_err());
    }

    #[test]
    fn bad_parse_headers() {
        let r = Request::from_reader(ChunkReader::new(
            "GET / HTTP/1.1\r\nHost localhost:42069\r\n\r\n",
            3,
        ));

        assert!(r.is_err());
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
