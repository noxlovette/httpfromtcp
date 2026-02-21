use std::num::NonZeroU16;

use crate::HTTPParsingError;

pub struct StatusCode(NonZeroU16);

impl StatusCode {
    pub fn from_bytes(src: &[u8]) -> Result<StatusCode, HTTPParsingError> {
        if src.len() != 3 {
            return Err(HTTPParsingError::BadStatusCode);
        }

        let a = src[0].wrapping_sub(b'0') as u16;
        let b = src[1].wrapping_sub(b'0') as u16;
        let c = src[2].wrapping_sub(b'0') as u16;

        if a == 0 || a > 9 || b > 9 || c > 9 {
            return Err(HTTPParsingError::BadStatusCode);
        }

        let status = (a * 100) + (b * 10) + c;
        NonZeroU16::new(status)
            .map(StatusCode)
            .ok_or_else(|| HTTPParsingError::BadStatusCode)
    }
}
