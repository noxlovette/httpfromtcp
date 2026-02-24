use httpfromtcp::{SERVER_PORT, Serve, shutdown_signal};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind(("127.0.0.1", SERVER_PORT)).await?;

    let signal = shutdown_signal();

    Serve::serve(listener, signal).await?;

    Ok(())
}
