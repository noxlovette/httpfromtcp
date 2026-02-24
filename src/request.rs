use crate::{HTTPParsingError, Headers, Method, ParserState, Version};
use std::fmt::{self};
use tokio::io::{AsyncRead, AsyncReadExt};

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
        for (k, v) in self.headers.0.iter() {
            writeln!(f, "– {}: {}", k, v)?;
        }

        Ok(())
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Head: {:?}", self.head)?;
        writeln!(f, "Body: {:?}", self.body)?;

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

    pub async fn from_reader(mut r: impl AsyncRead + Unpin) -> Result<Self, HTTPParsingError> {
        let mut req = Request::new();

        let mut buf = [0u8; 1024];
        let mut buf_len = 0;
        while !req.done() {
            let n = r.read(&mut buf[buf_len..]).await?;
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

                    if done {
                        self.state = ParserState::Done;
                    }

                    if n == 0 || current_data.len() == 0 {
                        break;
                    }

                    read += n;
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
    use std::{
        io,
        pin::Pin,
        task::{Context, Poll},
    };
    use tokio::io::{AsyncRead, ReadBuf};

    struct ChunkReader {
        data: Vec<u8>,
        num_bytes_per_read: usize,
        pos: usize,
    }

    impl ChunkReader {
        fn new(data: &str, num_bytes_per_read: usize) -> Self {
            Self {
                data: data.as_bytes().to_vec(),
                num_bytes_per_read,
                pos: 0,
            }
        }
    }

    impl AsyncRead for ChunkReader {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            if self.pos >= self.data.len() {
                return Poll::Ready(Ok(())); // EOF (no bytes appended)
            }

            let remaining = self.data.len() - self.pos;
            let allowed = self.num_bytes_per_read.min(remaining);
            let to_copy = allowed.min(buf.remaining());

            if to_copy == 0 {
                return Poll::Ready(Ok(()));
            }

            let end = self.pos + to_copy;
            buf.put_slice(&self.data[self.pos..end]);
            self.pos = end;

            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn good_get_request_line() {
        let r = Request::from_reader(ChunkReader::new(
            "GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n",
            3,
        ))
        .await
        .unwrap();

        assert_eq!("GET", r.head.method.as_str());
        assert_eq!("/", r.head.uri);
        assert_eq!("HTTP/1.1", r.head.version.as_str());
    }

    #[tokio::test]
    async fn good_get_request_line_with_path() {
        let r = Request::from_reader(ChunkReader::new(
            "GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n",
            1,
        ))
        .await
        .unwrap();

        assert_eq!("GET", r.head.method.as_str());
        assert_eq!("/coffee", r.head.uri);
        assert_eq!("HTTP/1.1", r.head.version.as_str());
    }

    #[tokio::test]
    async fn good_parse_headers() {
        let r = Request::from_reader(ChunkReader::new(
            "GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n ",
            3,
        ))
        .await
        .unwrap();

        let h = r.head.headers;
        assert_eq!("localhost:42069", h.get("host").unwrap());
        assert_eq!("curl/7.81.0", h.get("user-agent").unwrap());
        assert_eq!("*/*", h.get("accept").unwrap());
    }

    #[tokio::test]
    async fn good_parse_body() {
        let r = Request::from_reader(ChunkReader::new(
            "POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nContent-Length: 13\r\n\r\nhello world!\n",
            3,
        ))
        .await
        .unwrap();

        assert_eq!("hello world!\n", r.body);
    }

    #[tokio::test]
    async fn good_parse_empty_body_no_cl_no_body() {
        let r = Request::from_reader(ChunkReader::new(
            "POST /submit HTTP/1.1\r\nHost: localhost:42069\r\n\r\n\t\t",
            3,
        ))
        .await;

        assert!(r.is_ok())
    }

    #[tokio::test]
    async fn good_parse_empty_body_no_cl_empty_body() {
        let r = Request::from_reader(ChunkReader::new(
            "POST /submit HTTP/1.1\r\nHost: localhost:42069\r\n\r\n\t\t",
            3,
        ))
        .await;

        assert!(r.is_ok())
    }

    #[tokio::test]
    async fn bad_parse_body_shorter_content_length() {
        let r = Request::from_reader(ChunkReader::new(
            "POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nContent-Length: 20\r\n\r\nhello world!\n",
            3,
        ))
        .await;

        assert!(r.is_err());
    }

    #[tokio::test]
    async fn bad_parse_headers() {
        let r = Request::from_reader(ChunkReader::new(
            "GET / HTTP/1.1\r\nHost localhost:42069\r\n\r\n",
            3,
        ))
        .await;

        assert!(r.is_err());
    }

    #[tokio::test]
    async fn bad_input() {
        let r = Request::from_reader(ChunkReader::new(
            "/coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n",
            2,
        ))
        .await;

        assert!(r.is_err());
    }
}
