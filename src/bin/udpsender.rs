use std::{
    io::{BufRead, Stdin},
    net::UdpSocket,
    sync::mpsc,
    thread,
};

fn main() -> std::io::Result<()> {
    let s = UdpSocket::bind("0.0.0.0:0").unwrap();
    s.connect("127.0.0.1:42069").unwrap();
    let stdin = std::io::stdin();
    let rx = get_stdin(stdin);

    for line in rx {
        let b = line.as_bytes();
        s.send(b).expect("error sending line");
    }

    Ok(())
}

fn get_stdin(stdin: Stdin) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        let mut r = std::io::BufReader::new(stdin.lock());
        println!(">");
        loop {
            let mut line = String::new();
            let _n = match r.read_line(&mut line) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    println!("{e}");
                    break;
                }
            };

            tx.send(line.trim_end().to_string())
                .expect("error sending line");
        }
    });

    rx
}
