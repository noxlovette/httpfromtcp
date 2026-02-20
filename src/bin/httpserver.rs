use httpfromtcp::Serve;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    Ok(Serve::serve(None).await?)
}
