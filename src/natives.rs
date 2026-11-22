use std::io;
use std::rc::Rc;

use crate::environment::Environment;
use crate::value::{Native, Stash, Value};

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
    Native {
        name: "sayit",
        arity: 1,
        run: sayit,
    },
    Native {
        name: "size",
        arity: 1,
        run: size,
    },
    Native {
        name: "slice",
        arity: 3,
        run: slice,
    },
    Native {
        name: "has",
        arity: 2,
        run: has,
    },
    Native {
        name: "stash",
        arity: 0,
        run: stash,
    },
    Native {
        name: "stack",
        arity: 2,
        run: stack,
    },
    Native {
        name: "grab",
        arity: 2,
        run: grab,
    },
    Native {
        name: "put",
        arity: 3,
        run: put,
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

// sayit(value): the text spittin would print for the value, as a string. + never turns a
// number into text by itself, so this is how a program joins one onto a string: it asks, on
// purpose, and the conversion happens nowhere else. a string comes back as it went in
fn sayit(arguments: &[Value]) -> Result<Value, String> {
    Ok(Value::String(arguments[0].to_string()))
}

// size(text) or size(stash): how many characters a string has, or how many values a stash
// holds. characters, not bytes, so "é" is 1
fn size(arguments: &[Value]) -> Result<Value, String> {
    match &arguments[0] {
        Value::String(text) => Ok(Value::Number(text.chars().count() as f64)),
        Value::Stash(stash) => Ok(Value::Number(stash.items.borrow().len() as f64)),
        _ => Err("Argument to size must be a string or a stash.".to_string()),
    }
}

// slice(text, from, to): the characters from position `from` up to, but not including,
// position `to`. positions count from 0, so slice(text, 0, size(text)) is the whole string.
// bounds that don't fit are an error, not quietly cut down to fit: a program that asks for
// characters that aren't there has a bug, and handing back fewer would hide it
fn slice(arguments: &[Value]) -> Result<Value, String> {
    let Value::String(text) = &arguments[0] else {
        return Err("First argument to slice must be a string.".to_string());
    };
    let (Some(from), Some(to)) = (position(&arguments[1]), position(&arguments[2])) else {
        return Err("Slice positions must be whole numbers.".to_string());
    };
    let length = text.chars().count();
    if from > to || to > length {
        return Err(format!(
            "Can't slice {from} to {to} from a string of {length} characters."
        ));
    }
    Ok(Value::String(
        text.chars().skip(from).take(to - from).collect(),
    ))
}

// has(text, part): whether part appears anywhere in text. every string has the empty string
fn has(arguments: &[Value]) -> Result<Value, String> {
    match (&arguments[0], &arguments[1]) {
        (Value::String(text), Value::String(part)) => {
            Ok(Value::Boolean(text.contains(part.as_str())))
        }
        _ => Err("Arguments to has must be two strings.".to_string()),
    }
}

// a number that can name a position: whole and not negative. 1.5 and -1 name nothing, and
// rounding them would pick a position the program never asked for
fn position(value: &Value) -> Option<usize> {
    match value {
        Value::Number(number) if *number >= 0.0 && number.fract() == 0.0 && number.is_finite() => {
            Some(*number as usize)
        }
        _ => None,
    }
}

// stash(): a new, empty stash. every call makes a separate one
fn stash(_arguments: &[Value]) -> Result<Value, String> {
    Ok(Value::Stash(Rc::new(Stash::new())))
}

// stack(stash, value): puts value on the end of the stash. it changes the stash itself, which
// every name holding it will see, and answers nil: the change is the point, not a result
fn stack(arguments: &[Value]) -> Result<Value, String> {
    let stash = stash_argument(&arguments[0], "stack")?;
    stash.items.borrow_mut().push(arguments[1].clone());
    Ok(Value::Nil)
}

// grab(stash, position): the value at that position, counting from 0. a position with nothing
// there is an error, not nil: nil could be a value the program stacked, and the two must not
// look the same
fn grab(arguments: &[Value]) -> Result<Value, String> {
    let stash = stash_argument(&arguments[0], "grab")?;
    let items = stash.items.borrow();
    let index = stash_position(&arguments[1], items.len())?;
    Ok(items[index].clone())
}

// put(stash, position, value): replaces the value at a position that already holds one. it
// never grows the stash, so a typo'd position can't leave a gap of nothing behind it; stack
// is the only way to add
fn put(arguments: &[Value]) -> Result<Value, String> {
    let stash = stash_argument(&arguments[0], "put")?;
    let mut items = stash.items.borrow_mut();
    let index = stash_position(&arguments[1], items.len())?;
    items[index] = arguments[2].clone();
    Ok(Value::Nil)
}

// every stash native checks its first argument here, so the message names the native called
fn stash_argument<'a>(value: &'a Value, native: &str) -> Result<&'a Rc<Stash>, String> {
    match value {
        Value::Stash(stash) => Ok(stash),
        _ => Err(format!("First argument to {native} must be a stash.")),
    }
}

// a position that holds a value: whole, not negative, and less than the stash's size
fn stash_position(value: &Value, length: usize) -> Result<usize, String> {
    let Some(index) = position(value) else {
        return Err("Stash positions must be whole numbers.".to_string());
    };
    if index >= length {
        return Err(format!(
            "Position {index} is out of range for a stash of {length}."
        ));
    }
    Ok(index)
}
