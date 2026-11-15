use std::io;

use crate::environment::Environment;
use crate::value::{Native, Value};

// every native function the language has. adding one is a function below and a line here
const NATIVES: &[Native] = &[Native {
    name: "spill",
    arity: 0,
    run: spill,
}];

// the global scope a program starts in: empty except for the natives, which are bound before
// any user code runs. they are ordinary bindings, so a program can shadow or reassign one
pub fn globals() -> Environment {
    let environment = Environment::new();
    for native in NATIVES {
        environment.define(native.name, Value::Native(*native));
    }
    environment
}

// spill(): the next line of standard input as a string, without its line ending, or nil once
// the input has run out. an empty line is the empty string, which is truthy, so
// `spin (line = spill())` reads to the end of the input and stops there
fn spill(_arguments: &[Value]) -> Result<Value, String> {
    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(0) => Ok(Value::Nil),
        Ok(_) => {
            // only the line ending goes: spaces at either end may be meant
            let text = line.strip_suffix('\n').unwrap_or(&line);
            let text = text.strip_suffix('\r').unwrap_or(text);
            Ok(Value::String(text.to_string()))
        }
        Err(error) => Err(format!("Failed to read input: {error}.")),
    }
}
