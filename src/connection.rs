use crate::Encode;
use crate::Headers;
use crate::IntoResponse;
use crate::{Request, Response, ServerError};
use sha2::Digest;
use sha2::Sha256;
use std::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[derive(PartialEq, Default)]
pub enum ParserState {
    #[default]
    Init,
    Headers,
    Body,
    Done,
    Error,
}

pub struct Connection {
    io: TcpStream,
    req: Request,
    shutting_down: bool,
}

impl Connection {
    pub fn new(io: TcpStream) -> Self {
        Self {
            io,
            req: Request::new(),
            shutting_down: false,
        }
    }

    pub async fn run(&mut self) -> Result<(), ServerError> {
        self.read().await?;
        self.write().await?;
        Ok(())
    }

    pub async fn graceful_shutdown(&mut self) {
        self.shutting_down = true;
        if let Err(err) = self.io.shutdown().await {
            tracing::debug!("failed to close connection during shutdown: {err}");
        }
    }

    async fn read(&mut self) -> Result<(), ServerError> {
        if self.shutting_down {
            return Ok(());
        }
        self.req = Request::from_reader(&mut self.io).await?;
        tracing::info!("request received:\n {:?}", self.req);

        Ok(())
    }

    async fn write(&mut self) -> Result<(), ServerError> {
        if self.shutting_down {
            return Ok(());
        }

        let r = if self.req.head.uri.as_str() == "/myproblem" {
            Err(ServerError::Internal)
        } else if self.req.head.uri.as_str() == "yourproblem" {
            Err(ServerError::BadRequest)
        } else if self.req.head.uri.as_str().contains("/httpbin") {
            let bin = reqwest::get(
                self.req
                    .head
                    .uri
                    .as_str()
                    .replace("/httpbin", "https://httpbin.org"),
            )
            .await?;

            let mut body = Vec::<u8>::new();
            let bytes = bin.bytes().await.unwrap();

            for chunk in bytes.chunks(32) {
                body.extend_from_slice(format!("{:x}\r\n", chunk.len()).as_bytes());
                body.extend_from_slice(chunk);
                body.extend_from_slice("\r\n".as_bytes());
            }

            body.extend_from_slice("0\r\n".as_bytes());
            let sha = Sha256::digest(&bytes);

            let hex = hex::encode(sha);

            let mut trailers = Headers::new();

            trailers.set("X-Content-SHA256".to_string(), hex)?;
            trailers.set("X-Content-Length".to_string(), bytes.len().to_string())?;

            // suboptimal. the body should probably be Bytes, too
            let r = Response::new(Some(String::from_utf8(body).unwrap()))
                .chunked()
                .unwrap()
                .with_sha()
                .unwrap()
                .set_trailers(trailers);

            Ok(r)
        } else {
            let r = Response::new(Some(fs::read_to_string("200.html").unwrap()))
                .content_type("text/html")
                .unwrap();
            Ok(r)
        };

        tracing::info!("response generated:\n {r:?}");

        r.into_response().write(&mut self.io).await.unwrap();

        tracing::info!("response sent");

        Ok(())
    }
}
