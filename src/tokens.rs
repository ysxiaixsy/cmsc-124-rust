#[derive(Debug, Clone)]
pub enum Tokentypes {
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
    Comma,
    Identifier,
    Not,
    Slash,
    Star,
    Num,
    Dec,
    String,
    True,
    False,

    // keywords
    Ong,        // if
    Wait,       // else if
    Nah,        // else
    Spin,       // while / for
    Hold,       // let/var
    Locked,     // const
    Spittin,    // print
    Motion,     // function
    Pause,      // break
    Typeshi,    // return
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: Tokentypes,
    pub lexeme: String,
    pub line: usize,
}

