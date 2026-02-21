pub struct Method(Inner);
use std::fmt;

use crate::HTTPParsingError;

use self::Inner::*;

#[derive(PartialEq)]
enum Inner {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}
impl Default for Method {
    fn default() -> Self {
        Self(Get)
    }
}

impl PartialEq<str> for Method {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<&str> for Method {
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}

impl Method {
    pub const GET: Method = Method(Get);
    pub const POST: Method = Method(Post);
    pub const DELETE: Method = Method(Delete);
    pub const PUT: Method = Method(Put);
    pub const PATCH: Method = Method(Patch);

    pub fn from_bytes(src: &[u8]) -> Result<Method, HTTPParsingError> {
        match src.len() {
            0 => Err(HTTPParsingError::BadMethod),
            3 => match src {
                b"GET" => Ok(Method(Get)),
                b"PUT" => Ok(Method(Put)),
                _ => Err(HTTPParsingError::BadMethod),
            },
            4 => match src {
                b"POST" => Ok(Method(Post)),
                _ => Err(HTTPParsingError::BadMethod),
            },
            5 => match src {
                b"PATCH" => Ok(Method(Patch)),
                _ => Err(HTTPParsingError::BadMethod),
            },
            6 => match src {
                b"DELETE" => Ok(Method(Delete)),
                _ => Err(HTTPParsingError::BadMethod),
            },
            _ => Err(HTTPParsingError::BadMethod),
        }
    }

    pub fn as_str(&self) -> &str {
        match self.0 {
            Get => "GET",
            Post => "POST",
            Put => "PUT",
            Delete => "DELETE",
            Patch => "PATCH",
        }
    }
}

impl AsRef<str> for Method {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
