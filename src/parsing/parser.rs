use crate::lexing::token::{Delimiter, Keyword, Literal, Operator, Token, TokenType};

use super::node::*;

pub struct Parser {
    tokens: Vec<Token>, 
    current_index: usize,
    errors: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current_index: 0,
            errors: Vec::new(),
        }
    }
    fn advance(&mut self) -> Option<&Token> {
        self.current_index += 1;
        self.tokens.get(self.current_index - 1)
    }

    fn expect(&mut self, expected_type: TokenType) -> Result<&Token, String> {
        if let Some(token) = self.current_token() {
            if token.token_type == expected_type {
                return Ok(self.advance().unwrap());
            } else {
                // panic!("Expect function failed");
                let error_msg = format!(
                    "Expected token type {:?} but found {:?} at line {}, column {}",
                    expected_type, token.token_type, token.line, token.column
                );
                self.errors.push(error_msg.clone());
                Err(error_msg)
            }
        } else {
            let error_msg = "Unexpected end of file".to_string();
            self.errors.push(error_msg.clone());
            Err(error_msg)
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current_index + 1)
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.current_index)
    }
    fn is_at_end(&self) -> bool {
        self.current_index >= self.tokens.len()
    }

    fn synchronize(&mut self, expected_type: TokenType) {
        while !self.is_at_end() {
            if let Some(token) = self.current_token() {
                if token.token_type == expected_type {
                    break;
                }
            }
            self.advance();
        }
    }

    pub fn parse(&mut self) -> Result<ProgramNode, String> {
        let mut procedures = Vec::new();
        let mut main_procedure = None;

        while !self.is_at_end() {
            match self.parse_procedure() {
                Ok(proc) => {
                    if proc.name == "main" {
                        if main_procedure.is_some() {
                            let error_msg = "Multiple main procedures not allowed.".to_string();
                            self.errors.push(error_msg.clone());
                        } else if proc.return_type != TypeNode::VoidType {
                            let error_msg = "Main procedure cannot have a return type.".to_string();
                            self.errors.push(error_msg.clone());
                        }
                        main_procedure = Some(proc);
                    } else {
                        procedures.push(proc);
                    }
                }
                Err(_) => {
                    self.synchronize(TokenType::Keyword(Keyword::Procedure));
                }
            }
        }

        if self.errors.is_empty() {
            let main = match main_procedure {
                Some(m) => MainProcedureNode {
                    body: m.body,
                },
                None => {
                    return Err("Missing required main procedure".to_string())
                }
            };
            Ok(ProgramNode {
                procedures,
                main
            })
        } else {
            Err(self.errors.join("\n"))
        }
    }

    pub fn parse_procedure(&mut self) -> Result<ProcedureNode, String> {
        self.expect(TokenType::Keyword(Keyword::Procedure))?;
        let name = self.parse_identifier()?;
        let params = self.parse_parameters()?;
        let mut return_type = TypeNode::VoidType;
        if let Some(arrow) = self.current_token() {
            if let TokenType::Operator(Operator::Arrow) = arrow.token_type {
                self.advance();
                return_type = self.parse_type()?;
            }
        }
        let body = self.parse_block()?;

        Ok(ProcedureNode {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        match self.current_token() {
            Some(token) => match &token.token_type {
                TokenType::Identifier(_) => {
                    Ok(self.advance().unwrap().lexeme.clone())
                }

                _ => {
                    let error_msg = format!(
                        "Expected an identifier but found {:?} at line {}, column {}",
                        token.token_type, token.line, token.column
                    );
                    self.errors.push(error_msg.clone());
                    Err(error_msg)
                },
            },
            None => {
                let error_msg = "Unexpected end of file".to_string();
                self.errors.push(error_msg.clone());
                Err(error_msg)
            }
        }
    }

    pub fn parse_parameters(&mut self) -> Result<Vec<ParameterNode>, String> {
        let mut params = Vec::new();
        if let Some(token) = self.current_token() {
            if token.token_type != TokenType::Delimiter(Delimiter::LeftParenthesis) {
                return Ok(params);
            }
        }

        self.expect(TokenType::Delimiter(Delimiter::LeftParenthesis))?;

        while !self.current_token().map_or(false, |t| matches!(t.token_type, TokenType::Delimiter(Delimiter::RightParenthesis))) {
            let name = self.parse_identifier()?;
            self.expect(TokenType::Operator(Operator::Arrow))?;
            let param_type = self.parse_type()?;
            params.push(ParameterNode { name, param_type });

            if !self.is_at_end() && matches!(self.current_token().unwrap().token_type, TokenType::Delimiter(Delimiter::Comma)) {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(TokenType::Delimiter(Delimiter::RightParenthesis))?;
        Ok(params)
    }

    pub fn parse_type(&mut self) -> Result<TypeNode, String> {
        match self.current_token() {
            Some(token) => match &token.token_type {
                TokenType::Keyword(Keyword::NumType) => {
                    self.advance();
                    Ok(TypeNode::NumberType)
                }
                TokenType::Keyword(Keyword::BoolType) => {
                    self.advance();
                    Ok(TypeNode::BooleanType)
                }
                TokenType::Keyword(Keyword::StrType) => {
                    self.advance();
                    Ok(TypeNode::StringType)
                }
                _ => {
                    let error_msg = format!(
                        "Expected a type keyword but instead found '{:}' at line {}, column {}",
                        token.lexeme, token.line, token.column
                    );
                    self.errors.push(error_msg.clone());
                    Err(error_msg)
                }
            },
            None => {
                let error_msg = format!(
                    "Unexpected end of file while parsing type"
                );
                self.errors.push(error_msg.clone());
                Err(error_msg)
            }
        }
    }

    fn parse_block(&mut self) -> Result<BlockNode, String> {
        let mut statements = Vec::new();

        self.expect(TokenType::Delimiter(Delimiter::LeftBrace))?;

        while !self.is_at_end() {
            if let Some(token) = self.current_token() {
                if token.token_type == TokenType::Delimiter(Delimiter::RightBrace) {
                    break;
                }
                match self.parse_statement() {
                    Ok(statement) => {
                        statements.push(statement);
                    }
                    Err(_) => {
                        self.synchronize(TokenType::Delimiter(Delimiter::StatementEnd));
                        self.advance();
                    }
                }
            } else {
                break;
            }
        }

        self.expect(TokenType::Delimiter(Delimiter::RightBrace))?;

        Ok(BlockNode { statements })
    }

    fn parse_statement(&mut self) -> Result<StatementNode, String> {
        if self.is_at_end() {
            return Err("Unexpected end of input while parsing statement.".to_string());
        }

        match self.current_token() {
            Some(token) => match &token.token_type {
                TokenType::Keyword(Keyword::Leave) => {
                    self.advance();
                    self.expect(TokenType::Delimiter(Delimiter::StatementEnd))?;
                    Ok(StatementNode::Break)
                }
                TokenType::Keyword(Keyword::Repeat) => {
                    self.advance();
                    self.expect(TokenType::Delimiter(Delimiter::StatementEnd))?;
                    Ok(StatementNode::Continue)
                }
                TokenType::Keyword(Keyword::Define) => self.parse_variable_declaration(),
                TokenType::Identifier(_) => self.parse_assignment_or_expression(),
                TokenType::Keyword(Keyword::Yield) => self.parse_return(),
                TokenType::Keyword(Keyword::Loop) => self.parse_loop(),
                TokenType::Keyword(Keyword::When) => self.parse_conditional(),
                _ => {
                    let error_msg = format!(
                    "Unexpected token {:?} at line {}, column {}",
                    token.token_type, token.line, token.column
                    );
                    self.errors.push(error_msg.clone());
                    Err(error_msg)
                }
            },
            None => {
                let error_msg = format!(
                    "Unexpected end of file while parsing statement"
                );
                self.errors.push(error_msg.clone());
                Err(error_msg)
            }
        }
    }

    fn parse_assignment_or_expression(&mut self) -> Result<StatementNode, String> {
        if let Some(token) = self.peek() {
            if token.token_type == TokenType::Operator(Operator::Assign) {
                return self.parse_assignment();
            } else {
                let expression = self.parse_expression()?;
                self.expect(TokenType::Delimiter(Delimiter::StatementEnd))?;
                Ok(StatementNode::Expression(expression))
            }
        } else {
            let error_msg = format!(
                "unexpected end of file while parsing statement"
            );
            self.errors.push(error_msg.clone());
            Err(error_msg)
        }
    }
    
    fn parse_variable_declaration(&mut self) -> Result<StatementNode, String> {
        self.expect(TokenType::Keyword(Keyword::Define))?;

        let name = self.parse_identifier()?;

        let var_type = if let Some(Token { token_type: TokenType::Operator(Operator::Arrow), .. }) = self.current_token() {
            self.advance(); 
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenType::Operator(Operator::Assign))?;

        let initializer = self.parse_expression()?;
        self.expect(TokenType::Delimiter(Delimiter::StatementEnd))?;

        Ok(StatementNode::VariableDeclaration(VariableDeclarationNode {
            name,
            var_type,
            initializer,
        }))
    }

    fn parse_assignment(&mut self) -> Result<StatementNode, String> {
        let name = self.parse_identifier()?;

        self.expect(TokenType::Operator(Operator::Assign))?;

        let value = self.parse_expression()?;
        self.expect(TokenType::Delimiter(Delimiter::StatementEnd))?;

        Ok(StatementNode::Assignment(AssignmentNode {
            name,
            value,
        }))
    }

    fn parse_loop(&mut self) -> Result<StatementNode, String> {
        self.expect(TokenType::Keyword(Keyword::Loop))?;

        let body = self.parse_block()?;

        Ok(StatementNode::Loop(LoopNode {
            body,
        }))
    }
    fn parse_conditional(&mut self) -> Result<StatementNode, String> {
        self.expect(TokenType::Keyword(Keyword::When))?;

        let condition = self.parse_expression()?;
        let consequence = self.parse_block()?;

        let alternative = if let Some(Token { token_type: TokenType::Keyword(Keyword::Otherwise), .. }) = self.current_token() {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(StatementNode::Conditional(ConditionalNode {
            condition,
            consequence,
            alternative,
        }))
    }

    fn parse_return(&mut self) -> Result<StatementNode, String> {
        self.expect(TokenType::Keyword(Keyword::Yield))?;

        if let Some(next_token) = self.current_token() {
            if next_token.token_type == TokenType::Delimiter(Delimiter::StatementEnd) {
                self.advance();
                return Ok(StatementNode::Return(ReturnNode {
                    value: None,
                }));
            }
        } else {
            let error_msg = format!(
                "Unexpected end of file."
            );
            self.errors.push(error_msg.clone());
            return Err(error_msg);
        }

        let expression = self.parse_expression()?;
        self.expect(TokenType::Delimiter(Delimiter::StatementEnd))?;

        Ok(StatementNode::Return(ReturnNode {
            value: Some(expression),
        }))
    }

    


    fn parse_expression(&mut self) -> Result<ExpressionNode, String> {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, precedence: u8) -> Result<ExpressionNode, String> {
        let mut left = self.parse_unary_expression()?;
        while !self.is_at_end() {
            if let Some(token) = self.current_token() {
                let t = token.clone();
                if let TokenType::Operator(op) = t.token_type {
                    let op_precedence = op.get_precedence();
                    if op_precedence < precedence {
                        break;
                    }
                    self.advance();
                    let right = self.parse_binary_expression(op_precedence + 1)?;
                    left = ExpressionNode::BinaryOperation(Box::new(BinaryOperationNode  {
                        left,
                        operator: op.clone(),
                        right
                    }));
                } else {
                    break;
                }
            }
        }
        Ok(left)
    }

    fn parse_unary_expression(&mut self) -> Result<ExpressionNode, String> {
        if let Some(token) = self.current_token() {
            let t = token.clone();
            if let TokenType::Operator(op) = t.token_type {
                if let Operator::Minus = op {
                    self.advance();
                    let operand = self.parse_unary_expression()?;
                    return Ok(ExpressionNode::UnaryOperation(Box::new(UnaryOperationNode {
                        operator: op.clone(),
                        operand,
                    })));
                }
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<ExpressionNode, String> {
        if let Some(token) = self.current_token() {
            let t = token.clone();
            match t.token_type {
                TokenType::Identifier(name) => {
                    self.advance();
                    if let Some(TokenType::Delimiter(Delimiter::LeftParenthesis)) = self.current_token().map(|t| &t.token_type) {
                        return self.parse_procedure_call(name.clone());
                    }
                    Ok(ExpressionNode::Variable(name.clone()))
                }
                TokenType::Literal(Literal::NumberLiteral(value)) => {
                    self.advance();
                    Ok(ExpressionNode::Literal(LiteralNode { value: LiteralValue::NumberValue(value) }))
                }
                TokenType::Literal(Literal::StringLiteral(value)) => {
                    self.advance();
                    Ok(ExpressionNode::Literal(LiteralNode { value: LiteralValue::StringValue(value) }))
                }
                TokenType::Literal(Literal::BooleanLiteral(value)) => {
                    self.advance();
                    Ok(ExpressionNode::Literal(LiteralNode { value: LiteralValue::BooleanValue(value) }))
                }
                TokenType::Delimiter(Delimiter::LeftParenthesis) => {
                    self.advance();
                    let expr = self.parse_expression()?;
                    self.expect(TokenType::Delimiter(Delimiter::RightParenthesis))?;
                    Ok(expr)
                }
                _ => {
                    let error_msg = format!(
                        "Unexpected token {:?} at line {}, column {}",
                        t.token_type, t.line, t.column
                    );
                    self.errors.push(error_msg.clone());
                    Err(error_msg)
                }
            }
        } else {
            let error_msg = format!(
                "Unexpected end of file."
            );
            self.errors.push(error_msg.clone());
            Err(error_msg)
        }
    }

    fn parse_procedure_call(&mut self, name: String) -> Result<ExpressionNode, String> {
        self.expect(TokenType::Delimiter(Delimiter::LeftParenthesis))?;
        let mut args = Vec::new();

        if let Some(token) = self.current_token() {
            if token.token_type != TokenType::Delimiter(Delimiter::RightParenthesis) {
                loop {
                    args.push(self.parse_expression()?);
                    if let Some(TokenType::Delimiter(Delimiter::RightParenthesis)) = self.current_token().map(|t| &t.token_type) {
                        break;
                    }
                    self.expect(TokenType::Delimiter(Delimiter::Comma))?;
                }
            }
        }

        self.expect(TokenType::Delimiter(Delimiter::RightParenthesis))?;

        Ok(ExpressionNode::ProcedureCall(ProcedureCallNode { name, args }))
    }
}
