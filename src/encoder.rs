use crate::response::Parts;
use crate::{Headers, Response, ServerError, StatusCode, Version};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub trait Encode {
    fn write(&self, w: &mut TcpStream) -> impl Future<Output = Result<(), ServerError>>;
}

impl Encode for Response {
    async fn write(&self, w: &mut TcpStream) -> Result<(), ServerError> {
        self.head.write(w).await?;
        w.write_all(self.body.as_bytes()).await?;

        Ok(())
    }
}

impl Encode for Parts {
    async fn write(&self, w: &mut TcpStream) -> Result<(), ServerError> {
        self.status.write(w).await?;
        self.headers.write(w).await?;

        Ok(())
    }
}

impl Encode for StatusCode {
    async fn write(&self, w: &mut TcpStream) -> Result<(), ServerError> {
        match self {
            &StatusCode::OK => Ok(w.write_all("HTTP/1.1 200 OK\r\n".as_bytes()).await?),
            &StatusCode::BAD_REQUEST => Ok(w
                .write_all("HTTP/1.1 400 Bad Request\r\n".as_bytes())
                .await?),
            &StatusCode::INTERNAL_SERVER_ERROR => Ok(w
                .write_all("HTTP/1.1 500 Internal Server Error\r\n".as_bytes())
                .await?),
            _ => Err(ServerError::Internal),
        }
    }
}

impl Encode for Version {
    async fn write(&self, w: &mut TcpStream) -> Result<(), ServerError> {
        Ok(w.write_all(self.as_str().as_bytes()).await?)
    }
}

impl Encode for Headers {
    async fn write(&self, w: &mut TcpStream) -> Result<(), ServerError> {
        for (h, v) in &self.0 {
            w.write_all(format!("{}: {}\r\n", h, v).as_bytes()).await?;
        }

        w.write_all("\r\n".as_bytes()).await?;

        Ok(())
    }
}
