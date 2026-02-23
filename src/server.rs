use crate::{IntoResponse, Request, Response, SERVER_PORT, ServerError};
use tokio::net::TcpListener;

pub struct Serve {
    listener: TcpListener,
}

impl Serve {
    fn handler(req: &Request) -> impl IntoResponse {
        match req.head.uri.as_ref() {
            "/myproblem" => return Err(ServerError::Internal),
            "/yourproblem" => return Err(ServerError::BadRequest),
            _ => {
                let r = Response::new(Some("All good, frfr".to_string()));
                return Ok(r);
            }
        }
    }

    async fn run(self) -> Result<(), ServerError> {
        loop {
            let (mut io, remote_addr) = self.listener.accept().await?;

            println!("connection {remote_addr:?} accepted");

            let req = Request::from_reader_async(&io)?;
            println!("{:?}", req.head);

            tokio::spawn(async move {
                loop {
                    let r: Response = Self::handler(&req).into_response();
                    if let Err(e) = r.write(&mut io).await {
                        eprint!("{e}");
                    };
                    break;
                }
            });

            break;
        }

        Ok(self.close())
    }

    pub async fn serve(port: Option<u16>) -> Result<(), ServerError> {
        let listener = TcpListener::bind(("127.0.0.1", port.unwrap_or(SERVER_PORT))).await?;
        let server = Self { listener };
        server.run().await?;

        Ok(())
    }

    pub fn close(self) {
        drop(self.listener)
    }
}
