use std::net::TcpListener;

use httpfromtcp::Request;

fn main() -> Result<(), anyhow::Error> {
    let l = TcpListener::bind("127.0.0.1:42069").unwrap();

    match l.accept() {
        Ok((socket, addr)) => {
            println!("connection established from {addr}");

            let request = Request::from_reader(socket)?;

            println!("{}", request.request_line.unwrap());

            println!("file transfer complete. shutting down");
        }
        Err(e) => println!("couldn't get client: {e:?}"),
    }

    Ok(())
}
