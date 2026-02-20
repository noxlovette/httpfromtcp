use crate::{Headers, SERVER_PORT, ServerError, StatusCode};
use std::net::TcpListener;

pub struct Serve {
    listener: TcpListener,
}

impl Serve {
    async fn run(self) -> Result<(), ServerError> {
        loop {
            let (mut io, remote_addr) = self.listener.accept()?;

            println!("connection {remote_addr:?} accepted");

            tokio::spawn(async move {
                loop {
                    if let Err(e) = StatusCode::StatusOk.write(&mut io) {
                        eprintln!("{e}");
                        break;
                    }

                    if let Err(e) = Headers::default_headers(0).and_then(|h| h.write(&mut io)) {
                        eprintln!("{e}");
                        break;
                    }
                }
            });

            break;
        }

        Ok(self.close())
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
