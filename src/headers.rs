use crate::{HTTPParsingError, SEPARATOR};
use std::collections::HashMap;

pub struct Headers {
    pub headers: HashMap<String, String>,
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
                    break;
                }
                let (name, value) = Self::parse_header(&b[..i])?;

                read += i + SEPARATOR.len();

                self.headers.entry(name).insert_entry(value);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_parse() {
        let mut headers = Headers::new();

        let (n, done) = headers
            .parse("Host: localhost:42069\r\n\r\n".as_bytes())
            .unwrap();

        assert_eq!("localhost:42069", headers.headers["Host"]);
        assert_eq!(23, n);
        assert_eq!(done, true);
    }

    #[test]
    fn invalid_header() {
        let mut headers = Headers::new();

        let r = headers.parse("       Host : localhost:42069       \r\n\r\n".as_bytes());

        assert!(r.is_err())
    }
}
