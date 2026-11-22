use std::cell::RefCell;
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
    // an ordered collection of values. shared, like a function: binding a stash to a second
    // name, or passing it to a motion, hands over the same stash, so a change made through
    // one name is seen through the other
    Stash(Rc<Stash>),
}

// the values a stash holds, in the order they were stacked. the RefCell is what lets stack and
// put change a stash that several names share
pub struct Stash {
    pub items: RefCell<Vec<Value>>,
}

impl Stash {
    pub fn new() -> Self {
        Self {
            items: RefCell::new(Vec::new()),
        }
    }
}

// two stashes are equal only when they are the same stash, the same rule functions follow.
// comparing contents would also loop forever on a stash that holds itself
impl PartialEq for Stash {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

// written by hand for the same reason as Function's: a stash can hold itself
impl fmt::Debug for Stash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<stash of {}>", self.items.borrow().len())
    }
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
        write_value(self, f, &mut Vec::new())
    }
}

// a stash prints as its items in brackets: [1, "two", nocap]. a string inside one keeps its
// quotes, so ["1"] and [1] look different. `open` holds the stashes being printed right now,
// so one that holds itself prints [...] at that point instead of printing forever
fn write_value(
    value: &Value,
    f: &mut fmt::Formatter<'_>,
    open: &mut Vec<*const Stash>,
) -> fmt::Result {
    match value {
        Value::Number(value) => write!(f, "{value}"),
        Value::String(value) => write!(f, "{value}"),
        Value::Boolean(true) => write!(f, "nocap"),
        Value::Boolean(false) => write!(f, "cap"),
        Value::Nil => write!(f, "nil"),
        Value::Function(function) => write!(f, "{function:?}"),
        Value::Native(native) => write!(f, "{native:?}"),
        Value::Stash(stash) => {
            let address = Rc::as_ptr(stash);
            if open.contains(&address) {
                return write!(f, "[...]");
            }
            open.push(address);
            write!(f, "[")?;
            for (index, item) in stash.items.borrow().iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                match item {
                    Value::String(text) => write!(f, "{text:?}")?,
                    _ => write_value(item, f, open)?,
                }
            }
            open.pop();
            write!(f, "]")
        }
    }
}
