use std::iter::Peekable;
use std::str::FromStr;

use crate::lexing::{state_transition_table::State, token::*};


pub struct Lexer<I> 
where
    I: Iterator<Item = char>,
{
    input: Peekable<I>,
    current_state: State,
    buffer: String,
    current_line: usize,
    current_column: usize,
    last_char: Option<char>,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
{
    fn create_token(&self, token_type: TokenType) -> Token {
        Token::new(token_type, self.buffer.clone(), self.current_line, self.current_column - self.buffer.len())
    }

    pub fn new(input: I) -> Self {
        Lexer {
            input: input.peekable(),
            current_state: State::Start,
            buffer: String::new(),
            current_line: 1,
            current_column: 0,
            last_char: None
        }
    }
}

impl<I> Iterator for Lexer<I>
where
    I: Iterator<Item = char>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_state = State::Start;
        self.buffer.clear();
        while self.input.peek() != None {
            let c = if let Some(ch) = self.last_char { 
                self.last_char = None; 
                ch 
            } else { 
                if let Some(ch) = self.input.next() { ch } else { break; }
            };
            self.current_state = self.current_state.transition(c);
            if self.current_state == State::Start && self.buffer.is_empty() {
                if c == '\n' {
                    self.current_line += 1;
                    self.current_column = 0;
                }
                self.current_state = self.current_state.transition(c);
            }
            if c == '\n' {
                self.current_line += 1;
                self.current_column = 0;
            }
            self.current_column += 1;
            match self.current_state {
                State::Identifier => self.buffer.push(c),
                State::Number => self.buffer.push(c),
                State::String => self.buffer.push(c),
                State::Dot => self.buffer.push(c),
                State::MinusOrArrow => {
                    self.buffer.push(c);
                    if self.input.peek().is_none() {
                        return Some(self.create_token(TokenType::Operator(Operator::Minus)));
                    }
                    let token_type =  match self.input.peek().unwrap_or(&' ') {
                        '>' => {
                            self.buffer.push('>');
                            self.input.next();
                            self.current_column += 1;
                            TokenType::Operator(Operator::Arrow)
                        }
                        _ => TokenType::Operator(Operator::Minus),
                    };
                    return Some(self.create_token(token_type));
                },
                State::GtOrGe => {
                    self.buffer.push(c);
                    if self.input.peek().is_none() {
                        return Some(self.create_token(TokenType::Operator(Operator::Gt)));
                    }
                    let token_type =  match self.input.peek().unwrap_or(&' ') {
                        '=' => {
                            self.buffer.push('=');
                            self.input.next();
                            self.current_column += 1;
                            TokenType::Operator(Operator::Gte)
                        }
                        _ => TokenType::Operator(Operator::Gt),
                    };
                    return Some(self.create_token(token_type));
                },
                State::LtOrLe => {
                    self.buffer.push(c);
                    if self.input.peek().is_none() {
                        return Some(self.create_token(TokenType::Operator(Operator::Lt)));
                    }
                    let token_type =  match self.input.peek().unwrap() {
                        '=' => {
                            self.buffer.push('=');
                            self.input.next();
                            self.current_column += 1;
                            TokenType::Operator(Operator::Lte)
                        }
                        _ => TokenType::Operator(Operator::Lt),
                    };
                    return Some(self.create_token(token_type));
                },
                State::AssignOrError => {
                    self.buffer.push(c);
                    if self.input.peek().is_none() {
                        return Some(self.create_token(TokenType::Invalid));
                    }

                    let next_char = self.input.next().unwrap_or(' ');
                    self.buffer.push(next_char);
                    let token_type =  match next_char {
                        '=' => TokenType::Operator(Operator::Assign),
                        _ => TokenType::Invalid,
                    };
                    return Some(self.create_token(token_type));
                },
                State::Operator => {
                    self.buffer.push(c);
                    let token_type =  match c {
                        '+' => Some(TokenType::Operator(Operator::Plus)),
                        '*' => Some(TokenType::Operator(Operator::Times)),
                        '/' => Some(TokenType::Operator(Operator::Over)),
                        '%' => Some(TokenType::Operator(Operator::Mod)),
                        _ => Some(TokenType::Invalid)
                    };
                    if let Some(t) = token_type {
                        return Some(self.create_token(t));
                    }
                },
                State::Delimiter => {
                    self.buffer.push(c);
                    let token_type =  match c {
                        '{' => Some(TokenType::Delimiter(Delimiter::LeftBrace)),
                        '}' => Some(TokenType::Delimiter(Delimiter::RightBrace)),
                        '(' => Some(TokenType::Delimiter(Delimiter::LeftParenthesis)),
                        ')' => Some(TokenType::Delimiter(Delimiter::RightParenthesis)),
                        ',' => Some(TokenType::Delimiter(Delimiter::Comma)),
                        ';' => Some(TokenType::Delimiter(Delimiter::StatementEnd)),
                        _ => Some(TokenType::Invalid)
                    };
                    if let Some(t) = token_type {
                        return Some(self.create_token(t));
                    }
                }
                State::Comment => {
                    if !self.buffer.is_empty() {
                        self.last_char = Some(c);
                        let token_type = match TokenType::from_str(&self.buffer) {
                            Ok(tt) => tt,
                            Err(_) => TokenType::Invalid,
                        };
                        return Some(self.create_token(token_type));
                    }
                    while let Some(c) = self.input.next() {
                        if c == '\n' {
                            self.current_line += 1;
                            self.current_column = 0;
                            break;
                        }
                        self.current_column += 1;
                    } 
                    self.current_state = State::Start;
                }
                State::Whitespace => (),
                State::Invalid => {
                    self.buffer.push(c);
                    return Some(self.create_token(TokenType::Invalid));
                }
                State::Start => {
                    if !self.buffer.is_empty() {
                        if c == '"' {
                            self.last_char = None;
                            self.buffer.push('"');
                        } else {
                            self.last_char = Some(c);
                        }
                        let token_type = match TokenType::from_str(&self.buffer) {
                            Ok(tt) => tt,
                            Err(_) => TokenType::Invalid,
                        };
                        return Some(self.create_token(token_type));
                    } else {
                        self.buffer.push(c);
                    }
                }
            }
        }
        if !self.buffer.is_empty() {
            let token_type = match TokenType::from_str(&self.buffer) {
                Ok(tt) => tt,
                Err(_) => TokenType::Invalid,
            };
            return Some(self.create_token(token_type));
        }
        None
    }

}
