use crate::tokens::Token;

// one root owns every statement in source order
pub struct Program {
    pub statements: Vec<Stmt>,
}

pub enum Stmt {
    Expression(Expr),
    // spittin: the statement that makes a program's results observable
    Print(Expr),
    // hold: creates a binding. the name keeps its whole token so errors can report its line
    Hold { name: Token, initializer: Option<Expr> },
    // { ... }: its statements run in a new innermost scope that ends at the closing brace
    Block(Vec<Stmt>),
    // ong / wait / nah. the branches are statements, so either can be a block or another ong,
    // which is how `wait` needs no node of its own: the parser folds a chain into nested Ongs
    Ong {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    // spin: re-tests the condition before every pass, so a false condition runs the body zero times
    Spin { condition: Expr, body: Box<Stmt> },
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
    Variable(Token),
    Assign { name: Token, value: Box<Expr> },
}
