#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Identifier(String),
    Keyword(Keyword),
    Literal(Literal),
    Operator(Operator),
    Delimiter(Delimiter),
    Invalid,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    NumberLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Procedure, // `procedure`
    Define,    // `define`
    When,      // `when`
    Otherwise, // `otherwise`
    Loop,      // `loop`
    Yield,     // `yield`
    NumType,   // `number`
    BoolType,  // `boolean`
    StrType,   // `string`
    Leave,     // `break`
    Repeat,     // `repeat`
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Plus,        // +
    Minus,       // -
    Times,       // *
    Over,        // /
    Mod,         // %
    Lt,          // <
    Gt,          // >
    Lte,         // <=
    Gte,         // >=
    Arrow,       // `->`
    Assign,      // `:=`
    Is,          // `is`
    Isnt,        // `isnt`
    And,         // `and`
    Or,          // `or`
}

#[derive(Debug, PartialEq, Clone)]
pub enum Delimiter {
    LeftParenthesis,  // (
    RightParenthesis, // )
    LeftBrace,        // {
    RightBrace,       // }
    Comma,            // ,
    StatementEnd,     // ;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Token {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}

impl std::str::FromStr for TokenType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(n) = s.parse::<f64>() {
            Ok(TokenType::Literal(Literal::NumberLiteral(n)))
        } else if s.starts_with('"') && s.ends_with('"') {

            Ok(TokenType::Literal(Literal::StringLiteral(s[1..s.len() - 1].to_string())))
        } else if let Some(boolean) = match s {
            "yes" => Some(Literal::BooleanLiteral(true)),
            "no" => Some(Literal::BooleanLiteral(false)),
            _ => None,
        } {
            Ok(TokenType::Literal(boolean))
        } else if let Some(operator) = match s {
            "and" => Some(Operator::And),
            "or" => Some(Operator::Or),
            "is" => Some(Operator::Is),
            "isnt" => Some(Operator::Isnt),
            _ => None,
        } {
            Ok(TokenType::Operator(operator))
        } else if let Some(keyword) = match s {
            "procedure" => Some(Keyword::Procedure),
            "define" => Some(Keyword::Define),
            "when" => Some(Keyword::When),
            "otherwise" => Some(Keyword::Otherwise),
            "loop" => Some(Keyword::Loop),
            "leave" => Some(Keyword::Leave),
            "repeat" => Some(Keyword::Repeat),
            "yield" => Some(Keyword::Yield),
            "number" => Some(Keyword::NumType),
            "boolean" => Some(Keyword::BoolType),
            "string" => Some(Keyword::StrType),
            _ => None,
        } {
            Ok(TokenType::Keyword(keyword))
        } else {
            Ok(TokenType::Identifier(s.to_string()))
        }
    }
}



impl Operator {
    pub fn get_precedence(&self) -> u8 {
        match self {
            Operator::Or => 1,
            Operator::And => 2,

            Operator::Is => 3,
            Operator::Isnt => 3,

            Operator::Lt => 4,
            Operator::Gt => 4,
            Operator::Lte => 4,
            Operator::Gte => 4,

            Operator::Plus => 5,
            Operator::Minus => 5,
            Operator::Times => 6,
            Operator::Over => 6,
            Operator::Mod => 6,

            Operator::Assign => 0,
            Operator::Arrow => 0,
        }
    }
}
