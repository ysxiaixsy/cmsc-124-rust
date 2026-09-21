use crate::tokens::Token;

// the value a literal node holds; TrapScript has no nil yet
#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
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
