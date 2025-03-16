#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Start,
    Identifier,
    Number,
    String,
    Operator,
    Delimiter,
    Dot,
    Whitespace,

    MinusOrArrow,
    GtOrGe,
    LtOrLe,
    AssignOrError,

    Comment,

    Invalid,
}

impl State {
    pub fn transition(&self, c: char) -> State {
        match (self, c) {
            (State::Start, 'a'..='z' | 'A'..='Z') => State::Identifier,
            (State::Start, '0'..='9') => State::Number,
            (State::Start, '"') => State::String,
            (State::Start, '+' | '/' | '*' | '%') => State::Operator,
            (State::Start, '#') => State::Comment,
            (State::Start, '-') => State::MinusOrArrow,
            (State::Start, '<') => State::LtOrLe,
            (State::Start, '>') => State::GtOrGe,
            (State::Start, ':') => State::AssignOrError,
            (State::Start, '(' | ')' | '{' | '}' | ',' | ';') => State::Delimiter,
            (State::Start, ' ' | '\t' | '\n' | '\r') => State::Whitespace,
            (State::Start, _) => State::Invalid,

            (State::Identifier, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') => State::Identifier,
            (State::Identifier, _) => State::Start,

            (State::Number, '0'..='9') => State::Number,
            (State::Number, '.') => State::Dot,
            (State::Number, _) => State::Start,

            (State::String, '"') => State::Start,
            (State::String, _) => State::String,

            (State::Operator, _) => State::Start,

            (State::Delimiter, '(' | ')' | '{' | '}' | ',' | ';') => State::Delimiter,
            (State::Delimiter, _) => State::Invalid,

            (State::Dot, '0'..='9') => State::Number,
            (State::Dot, _) => State::Invalid,

            (State::Whitespace, ' ' | '\t' | '\n' | '\r') => State::Whitespace,
            (State::Whitespace, _) => State::Start,

            (State::LtOrLe, _) => State::Start,
            (State::GtOrGe, _) => State::Start,
            (State::MinusOrArrow, _) => State::Operator,
            (State::AssignOrError, _) => State::Invalid,
            (State::Comment, '\n') => State::Start,
            (State::Comment, _) => State::Comment,

            (State::Invalid, _) => State::Invalid,
        }
    }
}
