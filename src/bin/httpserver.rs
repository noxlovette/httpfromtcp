use httpfromtcp::Serve;

fn main() -> Result<(), anyhow::Error> {
    Ok(Serve::serve()?)
}
