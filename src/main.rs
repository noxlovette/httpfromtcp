use std::{fs::File, io::Read};

fn main() -> std::io::Result<()> {
    let mut f = File::open("messages.txt")?;
    let mut buf = [0u8; 8];

    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break; // EOF
        }

        let chunk = &buf[..n];

        match std::str::from_utf8(chunk) {
            Ok(s) => println!("{s}"),
            Err(e) => eprintln!("invalid utf-8: {e}"),
        }
    }

    Ok(())
}
