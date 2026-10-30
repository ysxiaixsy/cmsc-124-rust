use std::collections::HashMap;

use crate::evaluator::RuntimeError;
use crate::tokens::Token;
use crate::value::Value;

// the bindings a running program can see. one map is the single global scope; the same
// instance outlives each REPL line, which is how a session keeps its variables
pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    // hold: always binds here and never fails. inserting over an existing name is what makes
    // redeclaration legal, so this is the one line to change if that decision ever flips
    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        self.values
            .get(&name.lexeme)
            .cloned()
            .ok_or_else(|| undefined_variable(name))
    }

    // assignment only updates. it must never fall back to creating a binding, or every
    // misspelled name in a program would quietly become a new variable
    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), RuntimeError> {
        match self.values.get_mut(&name.lexeme) {
            Some(slot) => {
                *slot = value;
                Ok(())
            }
            None => Err(undefined_variable(name)),
        }
    }
}

// reading and assigning fail the same way, so their diagnostics cannot drift apart
fn undefined_variable(name: &Token) -> RuntimeError {
    RuntimeError::new(name, &format!("Undefined variable '{}'.", name.lexeme))
}
