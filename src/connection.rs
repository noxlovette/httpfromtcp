use crate::{Request, Response, ServerError};
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
    shutting_down: bool,
}

impl Connection {
    pub fn new(io: TcpStream) -> Self {
        Self {
            io,
            shutting_down: false,
        }
    }

    pub async fn run(&mut self) -> Result<(), ServerError> {
        self.read().await?;
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
        let req = Request::from_reader(&mut self.io).await?;

        tracing::info!("request received:\n {req:?}");

        let res = match req.head.uri.as_ref() {
            "/myproblem" => Err(ServerError::Internal),
            "/yourproblem" => Err(ServerError::BadRequest),
            _ => {
                let r = Response::new(Some(fs::read_to_string("200.html").unwrap()))
                    .content_type("text/html")
                    .unwrap();
                Ok(r)
            }
        };

        tracing::info!("response generated:\n {res:?}");

        if self.shutting_down {
            return Ok(());
        }

        // res.into_response().write(&mut stream).unwrap();
        // tracing::info!("response sent");

        Ok(())
    }
}
