use crate::tokens::Token;

// one root owns every statement in source order
pub struct Program {
    pub statements: Vec<Stmt>,
}

pub enum Stmt {
    Expression(Expr),
    // spittin: the statement that makes a program's results observable
    Print(Expr),
}

// the source value a literal node holds
#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

// one node of the syntax tree. operands are boxed because a node can't contain itself directly,
// only a pointer to another node. operators keep the whole token so Lab 3 can report its line
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Grouping(Box<Expr>),
    Unary { operator: Token, right: Box<Expr> },
    Binary { left: Box<Expr>, operator: Token, right: Box<Expr> },
}
