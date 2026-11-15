use std::io;

use crate::environment::Environment;
use crate::value::{Native, Value};

// every native function the language has. adding one is a function below and a line here
const NATIVES: &[Native] = &[
    Native {
        name: "spill",
        arity: 0,
        run: spill,
    },
    Native {
        name: "digits",
        arity: 1,
        run: digits,
    },
];

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

// digits(text): the number a string spells, or nil when it does not spell one. nil rather
// than an error, because the text usually came from spill and a program has to be able to
// check what it was given. nil in is nil out for the same reason: spill answers nil at the
// end of the input, so digits(spill()) has to survive it. any other value is a type error,
// as it is for every operator
fn digits(arguments: &[Value]) -> Result<Value, String> {
    // the evaluator has already checked the count, so the one argument is there
    let text = match &arguments[0] {
        Value::String(text) => text,
        Value::Nil => return Ok(Value::Nil),
        _ => return Err("Argument to digits must be a string.".to_string()),
    };

    // the spelling is the scanner's: digits, then at most one dot with any digits after it.
    // spaces around it are ignored, and a leading minus is allowed because input has no other
    // way to be negative. the host parser alone would also take .5, 1e3, +1, and inf
    let text = text.trim();
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let all_digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    if whole.is_empty() || !all_digits(whole) || !all_digits(fraction) {
        return Ok(Value::Nil);
    }

    match text.parse::<f64>() {
        // too many digits overflows to infinity, which the scanner rejects as well
        Ok(number) if number.is_finite() => Ok(Value::Number(number)),
        _ => Ok(Value::Nil),
    }
}
