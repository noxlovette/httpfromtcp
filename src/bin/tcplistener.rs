use httpfromtcp::{Request, SERVER_PORT};
use std::net::TcpListener;

fn main() -> Result<(), anyhow::Error> {
    let l = TcpListener::bind(("127.0.0.1", SERVER_PORT)).unwrap();

    match l.accept() {
        Ok((socket, addr)) => {
            println!("connection established from {addr}");

            let request = Request::from_reader(socket)?;

            println!("Head:");
            println!("{}", request.head);
            println!("Body:");
            println!("{}", request.body);
        }
        Err(e) => eprintln!("couldn't get client: {e:?}"),
    }

    Ok(())
}
