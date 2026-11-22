use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
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
    // the names in this scope that were declared with `locked`. a set beside the map, not a
    // flag on every value: locking belongs to the binding, and most bindings never have it
    locked: HashSet<String>,
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
            locked: HashSet::new(),
            enclosing,
        })))
    }

    // natives and parameters: always binds in the innermost scope and never fails. both go
    // into a scope that was just made, so there is nothing there yet to collide with
    pub fn define(&self, name: &str, value: Value) {
        self.0.borrow_mut().values.insert(name.to_string(), value);
    }

    // hold, motion, and locked: binds in the innermost scope without looking outward, so a
    // block's declaration shadows an outer name instead of touching it. inserting over an
    // existing name is what makes redeclaration in one scope legal. the one name it refuses
    // to replace is a locked one: if a second declaration could take its place, locking it
    // would have promised nothing. an enclosing scope's lock doesn't count, because a new
    // binding in here is a different binding
    pub fn declare(&self, name: &Token, value: Value, locked: bool) -> Result<(), RuntimeError> {
        let mut scope = self.0.borrow_mut();
        if scope.locked.contains(&name.lexeme) {
            let message = format!("Can't redeclare locked '{}'.", name.lexeme);
            return Err(RuntimeError::new(name, &message));
        }
        scope.values.insert(name.lexeme.clone(), value);
        if locked {
            scope.locked.insert(name.lexeme.clone());
        }
        Ok(())
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
    // binding, or every misspelled name in a program would quietly become a new variable. the
    // binding it finds is the one the name means here, so if that one is locked the assignment
    // is refused, rather than carrying on outward to some other binding of the same name
    pub fn assign(&self, name: &Token, value: Value) -> Result<(), RuntimeError> {
        let mut environment = self.clone();
        loop {
            let enclosing = {
                let mut scope = environment.0.borrow_mut();
                if scope.locked.contains(&name.lexeme) {
                    let message = format!("Can't assign to locked '{}'.", name.lexeme);
                    return Err(RuntimeError::new(name, &message));
                }
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
