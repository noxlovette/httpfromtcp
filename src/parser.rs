#[derive(PartialEq, Default)]
pub enum ParserState {
    #[default]
    Init,
    Done,
    Headers,
    Body,
    Error,
}
