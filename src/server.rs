use crate::{Connection, Listener, ServerError};
use core::pin::pin;
use std::net::SocketAddr;
use tokio::{net::TcpStream, signal, sync::watch};
use tracing::info;

pub struct Serve;

impl Serve {
    async fn handler<L>(
        io: TcpStream,
        signal_tx: &watch::Sender<()>,
        close_rx: &watch::Receiver<()>,
        remote_addr: SocketAddr,
    ) {
        tracing::info!("connection {remote_addr:?} accepted");

        let signal_tx = signal_tx.clone();
        let close_rx = close_rx.clone();

        tokio::spawn(async move {
            let mut conn = Connection::new(io);

            let mut signal_closed = pin!(signal_tx.closed());

            loop {
                tokio::select! {
                    result = conn.run() => {
                        if let Err(_err) = result {
                            tracing::error!("failed to serve connection: {_err:#}");
                        }
                        break;
                    }
                    _ = &mut signal_closed => {
                        tracing::info!("signal received in task, starting graceful shutdown");
                        conn.graceful_shutdown().await;
                        break;
                    }
                }
            }

            drop(close_rx);
        });
    }

    pub async fn serve<L, F>(mut listener: L, signal: F) -> Result<(), ServerError>
    where
        L: Listener<Io = TcpStream, Addr = SocketAddr>,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        let (signal_tx, signal_rx) = watch::channel(());

        tokio::spawn(async move {
            signal.await;
            info!("received graceful shutdown. telling tasks to shut down");
            drop(signal_rx);
        });

        let (close_tx, close_rx) = watch::channel(());

        loop {
            let (io, remote_addr) = tokio::select! {
                conn = listener.accept() => conn,
                _ = signal_tx.closed() => {
                    info!("signal received, not accepting new connections");
                    break;}
            };

            Self::handler::<L>(io, &signal_tx, &close_rx, remote_addr).await;
        }

        drop(close_rx);
        drop(listener);

        info!(
            "waiting for {} task(s) to finish",
            close_tx.receiver_count()
        );

        close_tx.closed().await;

        Ok(())
    }
}

pub async fn shutdown_signal() {
    let cc = async {
        signal::ctrl_c()
            .await
            .expect("failed to install ctrl+c handler")
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = cc => {},
        _ = terminate => {}
    }
}
