use crate::tokens::{LiteralValue, Token, Tokentypes};

pub struct ScanResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<String>,
    // how many tokens were scanned before the first error; None when there were no errors
    pub first_error_at: Option<usize>,
}





pub fn tokenizer(input: String) -> ScanResult {
    let mut errors: Vec<String> = Vec::new();
    let mut first_error_at: Option<usize> = None;
    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut line = 1;

    while i < chars.len() {
        let c = chars[i];

        match c {
            // whitespace
            ' ' | '\t' | '\r' => { i += 1; }
            '\n' => { line += 1; i += 1; }

            // single-char tokens with possible '=' lookahead
            '=' | '!' | '<' | '>' => {
                let next = chars.get(i + 1).copied();
                let (token_type, len) = match (c, next) {
                    ('=', Some('=')) => (Tokentypes::EqualEqual, 2),
                    ('=', _)         => (Tokentypes::Equal, 1),
                    ('!', Some('=')) => (Tokentypes::NotEqual, 2),
                    ('!', _)         => (Tokentypes::Not, 1),
                    ('<', Some('=')) => (Tokentypes::LessThanEqual, 2),
                    ('<', _)         => (Tokentypes::LessThan, 1),
                    ('>', Some('=')) => (Tokentypes::GreaterThanEqual, 2),
                    ('>', _)         => (Tokentypes::GreaterThan, 1),
                    _ => unreachable!(),
                };
                let lexeme: String = chars[i..i + len].iter().collect();
                tokens.push(Token { token_type, lexeme, literal: None, line });
                i += len;
            }

            // simple single-char tokens
            ',' => { tokens.push(Token { token_type: Tokentypes::Comma, lexeme: ",".into(), literal: None, line }); i += 1; }
            '+' => { tokens.push(Token { token_type: Tokentypes::Plus, lexeme: "+".into(), literal: None, line }); i += 1; }
            '-' => { tokens.push(Token { token_type: Tokentypes::Minus, lexeme: "-".into(), literal: None, line }); i += 1; }
            '*' => { tokens.push(Token { token_type: Tokentypes::Star, lexeme: "*".into(), literal: None, line }); i += 1; }
            '/' => {
                if chars.get(i + 1) == Some(&'/') {
                    i += 2;
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }
                } else {
                    tokens.push(Token { token_type: Tokentypes::Slash, lexeme: "/".into(), literal: None, line });
                    i += 1;
                }
            }
            '{' => { tokens.push(Token { token_type: Tokentypes::LeftBrace, lexeme: "{".into(), literal: None, line }); i += 1; }
            '}' => { tokens.push(Token { token_type: Tokentypes::RightBrace, lexeme: "}".into(), literal: None, line }); i += 1; }
            '(' => { tokens.push(Token { token_type: Tokentypes::LeftParen, lexeme: "(".into(), literal: None, line }); i += 1; }
            ')' => { tokens.push(Token { token_type: Tokentypes::RightParen, lexeme: ")".into(), literal: None, line }); i += 1; }

            // string literal
            '"' => {
                let start = i;
                let start_line = line;
                let mut value = String::new();
                i += 1; // skip opening quote
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    value.push(chars[i]);
                    i += 1;
                }
                if i >= chars.len() {
                    // get_or_insert only sets it the first time, so later errors don't move it
                    first_error_at.get_or_insert(tokens.len());
                    errors.push(format!("Unterminated string starting on line {}", start_line));
                    continue;
                }
                i += 1; // skip closing quote
                let lexeme = chars[start..i].iter().collect();
                tokens.push(Token { token_type: Tokentypes::String, lexeme, literal: Some(LiteralValue::String(value)), line: start_line });
            }

            // numbers (int or decimal)
            _ if c.is_ascii_digit() => {
                let mut lexeme = String::new();
                let mut is_decimal = false;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    if chars[i] == '.' {
                        if is_decimal {
                            break;
                        }
                        is_decimal = true;
                    }
                    lexeme.push(chars[i]);
                    i += 1;
                }
                let token_type = if is_decimal { Tokentypes::Dec } else { Tokentypes::Num };
                match lexeme.parse::<f64>() {
                    Ok(value) if value.is_finite() => {
                        tokens.push(Token { token_type, lexeme, literal: Some(LiteralValue::Number(value)), line });
                    }
                    _ => {
                        first_error_at.get_or_insert(tokens.len());
                        errors.push(format!("Invalid number '{}' on line {}", lexeme, line));
                    }
                }
            }

            // identifiers / keywords
            _ if c.is_alphabetic() || c == '_' => {
                let mut lexeme = String::new();
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    lexeme.push(chars[i]);
                    i += 1;
                }
                let token_type = match lexeme.as_str() {
                    "ong"     => Tokentypes::Ong,
                    "wait"    => Tokentypes::Wait,
                    "nah"     => Tokentypes::Nah,
                    "spin"    => Tokentypes::Spin,
                    "hold"    => Tokentypes::Hold,
                    "locked"  => Tokentypes::Locked,
                    "spittin" => Tokentypes::Spittin,
                    "motion"  => Tokentypes::Motion,
                    "pause"   => Tokentypes::Pause,
                    "typeshi" => Tokentypes::Typeshi, 
                    "true"    => Tokentypes::True,
                    "false"   => Tokentypes::False,
                    "nocap"   => Tokentypes::True,
                    "cap"     => Tokentypes::False,
                    _         => Tokentypes::Identifier,
                };
                let literal = match token_type {
                    Tokentypes::True => Some(LiteralValue::Boolean(true)),
                    Tokentypes::False => Some(LiteralValue::Boolean(false)),
                    _ => None,
                };
                tokens.push(Token { token_type, lexeme, literal, line });
            }

            _ => {
                first_error_at.get_or_insert(tokens.len());
                errors.push(format!(
                    "Unexpected character '{}' on line {}", c, line
                ));
                i += 1;
            }
        }
    }

    tokens.push(Token { token_type: Tokentypes::Eof, lexeme: String::new(), literal: None, line });
    ScanResult { tokens, errors, first_error_at }
}
