use httpfromtcp::get_lines_channel;
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let l = TcpListener::bind("127.0.0.1:42069").unwrap();

    match l.accept() {
        Ok((socket, addr)) => {
            println!("connection established from {addr}");

            let rx = get_lines_channel(socket);

            for line in rx {
                println!("{line}");
            }

            println!("file transfer complete. shutting down");
        }
        Err(e) => println!("couldn't get client: {e:?}"),
    }

    Ok(())
}
