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
    // a function the interpreter supplies, such as spill
    Native(Native),
}

// a function written in Rust instead of TrapScript. it is called exactly like a motion and
// its argument count is checked the same way, so a program cannot tell which kind it has
// except by printing it
#[derive(Clone, Copy)]
pub struct Native {
    pub name: &'static str,
    pub arity: usize,
    // a failure comes back as a message; the evaluator adds the line of the call
    pub run: fn(&[Value]) -> Result<Value, String>,
}

// every native has its own name, so the name is its identity
impl PartialEq for Native {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl fmt::Debug for Native {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native {}>", self.name)
    }
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
            Value::Native(native) => write!(f, "{native:?}"),
        }
    }
}
