use crate::io::PollBytes;
use std::task::{Context, Poll};

struct Decoder;

enum Kind {
    Length(u64),
    Eof(bool),
}
