use std::{env, fs, process};

#[derive(Debug, Clone)]
pub enum Tokens {

    // essential tokens
    Eof,
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Plus,
    Minus,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Identifier,
    Not,
    Slash,
    Star,
    Num,
    Dec,
    String,
    True,
    False,



}



fn fail(message: &str) -> ! {
    eprintln!("lab 0 error: {}", message);
    process::exit(65);
}

fn tokenizer (input: String) -> Vec<String> {
    // token vector for predicatble bytes 
    let mut tokens: Vec<String> = Vec::new();
    // current token to build until whitespace
    let mut current_token = String::new();
    //input chopped into chars to iterate over
    let chars = input.chars();
    for c in chars {
        if c.is_whitespace() {
            //if current token is not empty, push it to the tokens vector and clear it
            if !current_token.is_empty() {
                tokens.push(current_token.clone());
                current_token.clear();
            }
        } else {
            //build curr token until whitespace is found
            current_token.push(c);
        }
    }
    // if there's a remaining token, push it to the tokens vector
    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
}

fn main(){
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        fail("Usage: lab0 <source-file>");
    }

    let filepath = &args[1];

    let contents = fs::read_to_string(filepath).
    unwrap_or_else(|error| fail(&format!("Failed to read file: {}", error)));
    let tokens = tokenizer(contents);
    for token in tokens {
        println!("{}", token);
    }
}