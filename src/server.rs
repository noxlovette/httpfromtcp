use crate::{SERVER_PORT, ServerError};
use std::{io::Write, net::TcpListener};
use tokio::sync::watch;

pub struct Serve {
    listener: TcpListener,
}

impl Serve {
    async fn run(&self) -> Result<(), ServerError> {
        loop {
            let (mut io, remote_addr) = self.listener.accept()?;

            println!("connection {remote_addr:?} accepted");

            let message = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello World!";

            tokio::spawn(async move {
                loop {
                    if let Err(e) = io.write_all(message.as_bytes()) {
                        eprintln!("{e}");
                        break;
                    }
                }
            });
        }
    }

    pub async fn serve(port: Option<u16>) -> Result<(), ServerError> {
        let listener = TcpListener::bind(("127.0.0.1", port.unwrap_or(SERVER_PORT)))?;
        let server = Self { listener };
        server.run().await?;

        Ok(())
    }

    pub fn close(self) {
        drop(self.listener)
    }
}
