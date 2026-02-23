use std::io;
use std::str::Bytes;
use std::task::{Context, Poll};

pub trait PollBytes {
    fn read_mem(&mut self, cs: &mut Context<'_>, len: usize) -> Poll<io::Result<Bytes>>;
}
