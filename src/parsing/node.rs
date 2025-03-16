use crate::lexing::token::Operator;

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramNode {
    pub procedures: Vec<ProcedureNode>,
    pub main: MainProcedureNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureNode {
    pub name: String,
    pub params: Vec<ParameterNode>,
    pub return_type: TypeNode,
    pub body: BlockNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MainProcedureNode {
    pub body: BlockNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockNode {
    pub statements: Vec<StatementNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementNode {
    VariableDeclaration(VariableDeclarationNode),
    Assignment(AssignmentNode),
    Expression(ExpressionNode),
    Return(ReturnNode),
    Loop(LoopNode),
    Conditional(ConditionalNode),
    Break,
    Continue
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclarationNode {
    pub name: String,
    pub var_type: Option<TypeNode>,
    pub initializer: ExpressionNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentNode {
    pub name: String,
    pub value: ExpressionNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnNode {
    pub value: Option<ExpressionNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopNode {
    pub body: BlockNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalNode {
    pub condition: ExpressionNode,
    pub consequence: BlockNode,
    pub alternative: Option<BlockNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionNode {
    BinaryOperation(Box<BinaryOperationNode>),
    UnaryOperation(Box<UnaryOperationNode>),
    Literal(LiteralNode),
    Variable(String),
    ProcedureCall(ProcedureCallNode),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOperationNode {
    pub left: ExpressionNode,
    pub operator: Operator,
    pub right: ExpressionNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOperationNode {
    pub operator: Operator,
    pub operand: ExpressionNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteralNode {
    pub value: LiteralValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureCallNode {
    pub name: String,
    pub args: Vec<ExpressionNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterNode {
    pub name: String,
    pub param_type: TypeNode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeNode {
    NumberType,
    BooleanType,
    StringType,
    VoidType
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    NumberValue(f64),
    BooleanValue(bool),
    StringValue(String),
}
