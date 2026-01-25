use crate::{HTTPParsingError, SEPARATOR};
use std::fmt::{self, Write};
use std::{collections::HashMap, sync::LazyLock};

#[derive(Debug, Default)]
pub struct Headers {
    pub headers: HashMap<String, String>,
}

impl fmt::Display for Headers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Headers:")?;
        for (k, v) in self.headers.iter() {
            writeln!(f, "– {}: {}", k, v)?;
        }
        Ok(())
    }
}
impl Headers {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }
    pub fn parse(&mut self, b: &[u8]) -> Result<(usize, bool), HTTPParsingError> {
        let mut read: usize = 0;
        let mut done: bool = false;
        loop {
            if let Some(i) = b[read..]
                .windows(SEPARATOR.len())
                .position(|b| b == SEPARATOR)
            {
                if i == 0 {
                    // EMPTY HEADER
                    done = true;
                    read += SEPARATOR.len();
                    break;
                }

                let (name, value) = Self::parse_header(&b[read..read + i])?;

                is_token(&name)?;

                read += i + SEPARATOR.len();

                self.set(name, value)?;
            } else {
                break;
            }
        }

        Ok((read, done))
    }

    fn parse_header(field_line: &[u8]) -> Result<(String, String), HTTPParsingError> {
        let mut parts = field_line.splitn(2, |&b| b == b':');

        let name = parts
            .next()
            .filter(|n| !n.ends_with(b" "))
            .and_then(|name| std::str::from_utf8(name).ok())
            .ok_or(HTTPParsingError::BadFieldLine)?
            .to_string();

        let value = parts
            .next()
            .and_then(|v| std::str::from_utf8(v).ok())
            .ok_or(HTTPParsingError::BadFieldLine)?
            .trim()
            .to_string();

        Ok((name, value))
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_ascii_lowercase())
    }
    pub fn set(&mut self, mut name: String, value: String) -> Result<(), HTTPParsingError> {
        name = name.to_ascii_lowercase();

        if let Some(v) = self.headers.get(&name) {
            let mut s = String::new();
            write!(s, "{},{}", v, value)?;
            self.headers.insert(name, s);
        } else {
            self.headers.insert(name, value);
        }
        Ok(())
    }
}

fn is_token(str: &str) -> Result<(), HTTPParsingError> {
    let b = str.as_bytes();

    if b.is_empty() {
        return Err(HTTPParsingError::BadToken);
    }

    if !b.is_ascii() {
        return Err(HTTPParsingError::BadToken);
    }

    for &byte in b {
        if !LUT[byte as usize] {
            return Err(HTTPParsingError::BadToken);
        }
    }

    Ok(())
}

static LUT: LazyLock<[bool; 256]> = LazyLock::new(|| {
    let mut t = [false; 256];
    for b in b'a'..=b'z' {
        t[b as usize] = true
    }
    for b in b'A'..=b'Z' {
        t[b as usize] = true
    }

    for b in b'0'..=b'9' {
        t[b as usize] = true
    }
    for b in b"!#$%&'*+-.^_`|~" {
        t[*b as usize] = true
    }

    t
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_parse() {
        let mut headers = Headers::new();

        let (n, done) = headers
            .parse("Host: localhost:42069\r\n\r\n".as_bytes())
            .unwrap();

        assert_eq!("localhost:42069", headers.get("HOST").unwrap());
        assert_eq!(25, n);
        assert_eq!(done, true);
    }

    #[test]
    fn test_header_parse_double() {
        let mut headers = Headers::new();

        let (n, done) = headers
            .parse("Host: localhost:42069\r\nFooFoo:       barbar    \r\n\r\n".as_bytes())
            .unwrap();

        assert_eq!("localhost:42069", headers.get("HOST").unwrap());
        assert_eq!("barbar", headers.get("FooFoo").unwrap());
        assert_eq!(51, n);
        assert_eq!(done, true);
    }

    #[test]
    fn test_header_parse_multiple_value() {
        let mut headers = Headers::new();

        let (_, done) = headers
            .parse("Host: localhost:42069\r\nHost: localhost:42068\r\n\r\n".as_bytes())
            .unwrap();

        assert_eq!(
            "localhost:42069,localhost:42068",
            headers.get("HOST").unwrap()
        );
        assert_eq!(done, true);
    }

    #[test]
    fn invalid_header() {
        let mut headers = Headers::new();

        let r = headers.parse("       Host : localhost:42069       \r\n\r\n".as_bytes());

        assert!(r.is_err())
    }

    #[test]
    fn invalid_token() {
        let mut headers = Headers::new();

        let r = headers.parse("       H@st : localhost:42069       \r\n\r\n".as_bytes());

        assert!(r.is_err())
    }
}
