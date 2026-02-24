use std::{net::SocketAddr, time::Duration};

use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
};

pub trait Listener: Send + 'static {
    type Io: AsyncRead + AsyncWrite + Unpin + Send + 'static;

    type Addr: Send;

    fn accept(&mut self) -> impl Future<Output = (Self::Io, Self::Addr)> + Send;
}

impl Listener for TcpListener {
    type Io = TcpStream;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            match Self::accept(self).await {
                Ok(tup) => return tup,
                Err(e) => handle_error(e).await,
            }
        }
    }
}

pub async fn handle_error(e: io::Error) {
    if is_connection_error(&e) {
        return;
    }

    tracing::error!("accept error: {e}");

    tokio::time::sleep(Duration::from_secs(1)).await;
}

fn is_connection_error(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset
    )
}
