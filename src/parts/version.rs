use crate::HTTPParsingError;
use std::fmt;

#[derive(Default)]
pub struct Version(Http);

impl Version {
    pub const HTTP_11: Version = Version(Http::Http11);
}

#[derive(Default)]
enum Http {
    #[default]
    Http11,
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Http::Http11;

        f.write_str(match self.0 {
            Http11 => "HTTP/1.1",
        })
    }
}

impl Version {
    pub fn as_str(&self) -> &str {
        match self.0 {
            Http::Http11 => "1.1",
        }
    }
    pub fn from_bytes(src: &[u8]) -> Result<Self, HTTPParsingError> {
        let mut parts = src.split(|&b| b == b'/');

        parts
            .next()
            .filter(|p| p == b"HTTP")
            .ok_or_else(|| HTTPParsingError::BadRequestLine)?;

        let version = parts
            .next()
            .ok_or_else(|| HTTPParsingError::BadRequestLine)?;

        if parts.next().is_some() {
            return Err(HTTPParsingError::BadRequestLine);
        }

        match version {
            b"1.1" => Ok(Version::HTTP_11),
            _ => Err(HTTPParsingError::UnsupportedHTTPVersion),
        }
    }
}
