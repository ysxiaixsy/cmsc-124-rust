use std::collections::HashMap;

use crate::evaluator::RuntimeError;
use crate::tokens::Token;
use crate::value::Value;

// the bindings a running program can see: one scope's map, plus the scope that textually
// encloses it. the outermost scope is global, and the same instance outlives each REPL line,
// which is how a session keeps its variables. lookup follows these links, never the order of
// calls, so scope stays static once functions arrive
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    // entering a block: a fresh innermost scope whose parent is the scope that was current
    pub fn push(&mut self) {
        let outer = std::mem::replace(self, Environment::new());
        self.enclosing = Some(Box::new(outer));
    }

    // leaving a block: its bindings are dropped and the enclosing scope is current again.
    // popping the global scope does nothing, so a mismatched pop can't lose the session
    pub fn pop(&mut self) {
        if let Some(outer) = self.enclosing.take() {
            *self = *outer;
        }
    }

    // hold: always binds in the innermost scope and never fails, so a block's hold shadows an
    // outer name instead of touching it. inserting over an existing name is what makes
    // redeclaration in one scope legal, so this is the one line to change if that ever flips
    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    // innermost scope first, then outward; the first match wins and hides any outer one
    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        let mut scope = Some(self);
        while let Some(environment) = scope {
            if let Some(value) = environment.values.get(&name.lexeme) {
                return Ok(value.clone());
            }
            scope = environment.enclosing.as_deref();
        }
        Err(undefined_variable(name))
    }

    // assignment searches the same way, and only updates. it must never fall back to creating a
    // binding, or every misspelled name in a program would quietly become a new variable
    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), RuntimeError> {
        let mut environment = self;
        loop {
            if let Some(slot) = environment.values.get_mut(&name.lexeme) {
                *slot = value;
                return Ok(());
            }
            match environment.enclosing.as_deref_mut() {
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
