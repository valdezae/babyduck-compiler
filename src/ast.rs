// AST definitions for the BabyDuck language

#[derive(Debug, Clone)]
pub struct Program {
    pub id: String,
    pub vars: Vec<VarDeclaration>,
    pub funcs: Vec<FunctionDeclaration>,
    pub main_body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct VarDeclaration {
    pub id: String,
    pub var_type: Type,
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment(Assignment),
    Condition(Condition),
    Cycle(Cycle),
    FunctionCall(FunctionCall),
    Print(PrintStatement),
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub id: String,
    pub expression: Expression,
}

#[derive(Debug, Clone)]
pub enum Expression {
    BinaryOp {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    },
    Identifier(String),
    IntegerLiteral(i32),
    FloatLiteral(f64),
}

#[derive(Debug, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    GreaterThan,
    LessThan,
    Equal,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub condition: Expression,
    pub if_body: Vec<Statement>,
    pub else_body: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct Cycle {
    pub condition: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub id: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub enum PrintStatement {
    Expression(Expression),
    StringLiteral(String),
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub id: String,
    pub parameters: Vec<Parameter>,
    pub vars: Vec<VarDeclaration>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub id: String,
    pub param_type: Type,
}