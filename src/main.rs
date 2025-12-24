use std::{fs::File, io::Read};

fn main() -> std::io::Result<()> {
    let mut f = File::open("messages.txt")?;
    let mut buf = [0u8; 8];

    let mut str = String::new();
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break; // EOF
        }

        let mut chunk = &buf[..n];

        if let Some(i) = chunk.iter().position(|&b| b == b'\n') {
            str += std::str::from_utf8(&chunk[..i]).expect("utf");
            println!("read: {str}");

            chunk = &chunk[i + 1..];
            str = String::new();
        }

        str += std::str::from_utf8(chunk).expect("utf error");
    }

    Ok(())
}
