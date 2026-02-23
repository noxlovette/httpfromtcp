use crate::{Encode, IntoResponse, Request, Response, SERVER_PORT, ServerError, ThreadPool};
use std::net::{TcpListener, TcpStream};

pub struct Serve;

impl Serve {
    fn handler(mut stream: TcpStream) {
        let req = Request::from_reader(&stream).unwrap();
        println!("request received");
        let res = match req.head.uri.as_ref() {
            "/myproblem" => Err(ServerError::Internal),
            "/yourproblem" => Err(ServerError::BadRequest),
            _ => {
                let r = Response::new(Some("All good, frfr".to_string()));
                Ok(r)
            }
        };

        res.into_response().write(&mut stream).unwrap();
        println!("response sent");
    }

    fn run(self) -> Result<(), ServerError> {
        let listener = TcpListener::bind(("127.0.0.1", SERVER_PORT))?;

        let pool = ThreadPool::new(4);

        for stream in listener.incoming().take(2) {
            let stream = stream?;

            pool.execute(|| {
                Self::handler(stream);
            });
        }

        drop(pool);

        Ok(())
    }

    pub fn serve() -> Result<(), ServerError> {
        let server = Self;
        server.run()?;

        Ok(())
    }
}
