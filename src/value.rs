use std::fmt;
use std::rc::Rc;

use crate::ast::FunctionDecl;
use crate::environment::Environment;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    // shared, so binding a function to a second name copies the reference, not the function
    Function(Rc<Function>),
}

// what running a `motion` declaration produces: the declaration, plus the scope it ran in.
// a call reads names through `closure`, never through whoever called it, which is what keeps
// scope static
pub struct Function {
    pub declaration: Rc<FunctionDecl>,
    pub closure: Environment,
}

// two functions are equal only when they are the same function value. two declarations with
// identical text are still different functions, and so are two runs of one declaration
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

// written by hand: a function's scope usually holds the function itself, so a derived Debug
// would follow that loop forever
impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<motion {}>", self.declaration.name.lexeme)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(value) => write!(f, "{value}"),
            Value::String(value) => write!(f, "{value}"),
            Value::Boolean(true) => write!(f, "nocap"),
            Value::Boolean(false) => write!(f, "cap"),
            Value::Nil => write!(f, "nil"),
            Value::Function(function) => write!(f, "{function:?}"),
        }
    }
}
