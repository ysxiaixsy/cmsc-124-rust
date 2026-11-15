use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::evaluator::RuntimeError;
use crate::tokens::Token;
use crate::value::Value;

// the bindings a running program can see: one scope's map, plus the scope that textually
// encloses it. the outermost scope is global, and the same one outlives each REPL line, which
// is how a session keeps its variables. lookup follows these links, never the order of calls,
// so scope stays static.
//
// an Environment is a handle: cloning it gives a second handle to the same scope, not a copy.
// a function keeps a handle to the scope it was declared in, so that scope has to be able to
// outlive the block that created it. that is why scopes are reference-counted instead of
// being pushed and popped
#[derive(Clone)]
pub struct Environment(Rc<RefCell<Scope>>);

struct Scope {
    values: HashMap<String, Value>,
    enclosing: Option<Environment>,
}

impl Environment {
    // the global scope: it has nothing around it
    pub fn new() -> Self {
        Self::with_enclosing(None)
    }

    // entering a block or a call: a fresh innermost scope around which this one sits. when the
    // last handle to it goes, its bindings go with it
    pub fn child(&self) -> Self {
        Self::with_enclosing(Some(self.clone()))
    }

    fn with_enclosing(enclosing: Option<Environment>) -> Self {
        Self(Rc::new(RefCell::new(Scope {
            values: HashMap::new(),
            enclosing,
        })))
    }

    // hold: always binds in the innermost scope and never fails, so a block's hold shadows an
    // outer name instead of touching it. inserting over an existing name is what makes
    // redeclaration in one scope legal, so this is the one line to change if that ever flips
    pub fn define(&self, name: &str, value: Value) {
        self.0.borrow_mut().values.insert(name.to_string(), value);
    }

    // innermost scope first, then outward; the first match wins and hides any outer one
    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        let mut environment = self.clone();
        loop {
            let enclosing = {
                let scope = environment.0.borrow();
                if let Some(value) = scope.values.get(&name.lexeme) {
                    return Ok(value.clone());
                }
                scope.enclosing.clone()
            };
            match enclosing {
                Some(outer) => environment = outer,
                None => return Err(undefined_variable(name)),
            }
        }
    }

    // assignment searches the same way, and only updates. it must never fall back to creating a
    // binding, or every misspelled name in a program would quietly become a new variable
    pub fn assign(&self, name: &Token, value: Value) -> Result<(), RuntimeError> {
        let mut environment = self.clone();
        loop {
            let enclosing = {
                let mut scope = environment.0.borrow_mut();
                if let Some(slot) = scope.values.get_mut(&name.lexeme) {
                    *slot = value;
                    return Ok(());
                }
                scope.enclosing.clone()
            };
            match enclosing {
                Some(outer) => environment = outer,
                None => return Err(undefined_variable(name)),
            }
        }
    }
}

// reading and assigning fail the same way, so their diagnostics cannot drift apart
fn undefined_variable(name: &Token) -> RuntimeError {
    RuntimeError::new(name, &format!("Undefined variable '{}'.", name.lexeme))
}
