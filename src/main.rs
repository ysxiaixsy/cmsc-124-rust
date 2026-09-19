mod tokens;
mod tokenizer;

use std::{env, fs, process};
use tokenizer::tokenizer;

fn fail(message: &str) -> ! {
    eprintln!("lab 0 error: {}", message);
    process::exit(65);
}



fn main(){
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 3 || args[1] != "--tokenize" {
        fail("Usage: ./run --tokenize <source-file>");
    }

    let filepath = &args[2];

    let contents = fs::read_to_string(filepath).
    unwrap_or_else(|error| fail(&format!("Failed to read file: {}", error)));
    let result = tokenizer(contents);

    if !result.errors.is_empty(){
        for error in result.errors {
            eprintln!("Error: {}", error);
        }
        process::exit(65)
    }
    // should print token stream
    for token in result.tokens {
        println!("{:#?}", token);
    }
}