use std::{io::Read, net::TcpListener, sync::mpsc, thread};

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

fn get_lines_channel<R>(mut r: R) -> mpsc::Receiver<String>
where
    R: Read + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        let mut buf = [0u8; 8];

        let mut str = String::new();
        loop {
            let n = match r.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    print!("{e}");
                    break;
                }
            };

            let mut chunk = &buf[..n];

            if let Some(i) = chunk.iter().position(|&b| b == b'\n') {
                str += std::str::from_utf8(&chunk[..i]).expect("utf");
                let _ = tx.send(str);

                chunk = &chunk[i + 1..];
                str = String::new();
            }

            str += std::str::from_utf8(chunk).expect("utf error");
        }
    });

    rx
}
