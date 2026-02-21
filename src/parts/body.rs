use crate::{HTTPParsingError, Request};

impl Request {
    pub fn parse_body(&mut self, b: &[u8]) -> Result<(usize, bool), HTTPParsingError> {
        let (mut read, mut done) = (0, false);

        if let Some(cl) = self.head.headers.get("content-length") {
            let n: usize = cl.parse()?;
            let remaining = n - self.body.len();
            let m = remaining.min(b.len());
            let s = std::str::from_utf8(&b[..m])?;
            self.body += s;

            read += m;

            if self.body.len() == n {
                done = true;
            }
        } else {
            done = true;
        }

        Ok((read, done))
    }
}
