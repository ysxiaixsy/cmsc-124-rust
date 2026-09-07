use std::{env, fs, process};

#[derive(Debug, Clone)]
pub enum Tokentypes {

    // essential tokens
    Eof,
    Equal,
    EqualEqual,
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
    Pipeline,
}

#[derive(Debug, Clone)]
pub enum Keywords {
    Ong,        // if
    Wait,       // else if
    Nah,        // else

    Spin,       // while (condition) / for (condition)

    Hold,       // let/var
    Locked,     // const

    Spittin,    // print
    Motion,     // function
    Pause,      // break
    Typeshi,    // return   
}
#[derive(Debug, Clone)]
struct Token {
    token_type: Tokentypes,
    lexeme: String,
    line: usize,
}


fn fail(message: &str) -> ! {
    eprintln!("lab 0 error: {}", message);
    process::exit(65);
}

fn tokenizer (input: String) -> Vec<Token> {
    // token vector for predicatble bytes 
    let mut tokens: Vec<Token> = Vec::new();
    // current token to build until whitespace
    let mut current_token = String::new();

    let mut line = 1; // initialize line number

    //input chopped into chars to iterate over
    let chars = input.chars();
    for c in chars {
        if c.is_whitespace() {
            //if current token is not empty, push it to the tokens vector and clear it
            if !current_token.is_empty() {
                tokens.push(Token {
                    token_type: Tokentypes::Identifier,
                    lexeme: current_token.clone(),
                    line: line,
                });
                current_token.clear();
            }

            // += line count when newline is found
            if c == '\n' {
                line += 1;
            }

        } else {
            //build curr token until whitespace is found
            current_token.push(c);
        }
    }
    
    // if there's a remaining token, push it to the tokens vector
    if !current_token.is_empty() {
        tokens.push(Token {
            token_type: Tokentypes::Identifier,
            lexeme: current_token.clone(),
            line: line,
        });
    }

    // eof token
    tokens.push(Token {
        token_type: Tokentypes::Eof,
        lexeme: String::new(),
        line: line,
    });

    tokens
}

fn main(){
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 3 || args[1] != "--tokenize" {
        fail("Usage: ./run --tokenize <source-file>");
    }

    let filepath = &args[2];

    let contents = fs::read_to_string(filepath).
    unwrap_or_else(|error| fail(&format!("Failed to read file: {}", error)));
    let tokens = tokenizer(contents);

    // should print token stream
    for token in tokens {
        println!("{:#?}", token);
    }
}