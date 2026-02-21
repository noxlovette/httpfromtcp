#[derive(PartialEq, Default)]
pub enum ParserState {
    #[default]
    Init,
    Headers,
    Body,
    Done,
    Error,
}
