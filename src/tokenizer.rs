use crate::tokens::{Token, Tokentypes};

pub struct ScanResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<String>,
}





pub fn tokenizer(input: String) -> ScanResult {
    let mut errors: Vec<String> = Vec::new();
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
                tokens.push(Token { token_type, lexeme, line });
                i += len;
            }

            // simple single-char tokens
            ',' => { tokens.push(Token { token_type: Tokentypes::Comma, lexeme: ",".into(), line }); i += 1; }
            '+' => { tokens.push(Token { token_type: Tokentypes::Plus, lexeme: "+".into(), line }); i += 1; }
            '-' => { tokens.push(Token { token_type: Tokentypes::Minus, lexeme: "-".into(), line }); i += 1; }
            '*' => { tokens.push(Token { token_type: Tokentypes::Star, lexeme: "*".into(), line }); i += 1; }
            '/' => { tokens.push(Token { token_type: Tokentypes::Slash, lexeme: "/".into(), line }); i += 1; }
            '{' => { tokens.push(Token { token_type: Tokentypes::LeftBrace, lexeme: "{".into(), line }); i += 1; }
            '}' => { tokens.push(Token { token_type: Tokentypes::RightBrace, lexeme: "}".into(), line }); i += 1; }
            '(' => { tokens.push(Token { token_type: Tokentypes::LeftParen, lexeme: "(".into(), line }); i += 1; }
            ')' => { tokens.push(Token { token_type: Tokentypes::RightParen, lexeme: ")".into(), line }); i += 1; }

            // string literal
            '"' => {
                let start_line = line;
                let mut lexeme = String::new();
                i += 1; // skip opening quote
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    lexeme.push(chars[i]);
                    i += 1;
                }
                if i >= chars.len() {
                    errors.push(format!("Unterminated string starting on line {}", start_line));
                }
                i += 1; // skip closing quote
                tokens.push(Token { token_type: Tokentypes::String, lexeme, line: start_line });
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
                tokens.push(Token { token_type, lexeme, line });
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
                tokens.push(Token { token_type, lexeme, line });
            }

            _ => {
                errors.push(format!(
                    "Unexpected character '{}' on line {}", c, line
                ));
                i += 1;
            }
        }
    }

    tokens.push(Token { token_type: Tokentypes::Eof, lexeme: String::new(), line });
    ScanResult { tokens, errors }
}