use crate::Frame;
use bytes::Bytes;
use std::{
    io,
    task::{Context, Poll},
};
use tracing::info;

pub struct Decoder {
    kind: Kind,
}

#[derive(Debug)]
enum Kind {
    Length(u64),
    Eof(bool),
}

impl Decoder {
    pub fn length(x: u64) -> Self {
        Self {
            kind: Kind::Length(x),
        }
    }

    pub fn eof() -> Self {
        Self {
            kind: Kind::Eof(false),
        }
    }

    pub fn decode(&mut self, cx: &mut Context<'_>) -> Poll<Result<Frame<Bytes>, io::Error>> {
        info!("decode; state={:?}", self.kind);
        match self.kind {
            Kind::Length(ref mut remaining) => {
                if *remaining == 0 {
                    Poll::Ready(Ok(Frame::data(Bytes::new())));
                } else {
                    let to_read = *remaining as usize;
                }
            }
            Kind::Eof(ref mut is_eof) => {}
        }
        todo!()
    }
}
