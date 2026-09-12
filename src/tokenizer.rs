use crate::tokens::{Token, Tokentypes};


pub fn tokenizer (input: String) -> Vec<Token> {
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