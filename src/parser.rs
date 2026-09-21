use crate::ast::{Expr, Literal};
use crate::tokens::{Token, Tokentypes};

// a syntax error, already formatted with its line: [line 1] Error at ')': Expect expression.
pub struct ParseError {
    pub message: String,
}

// recursive descent: one function per grammar rule, each calling the rule below it.
// the only state is the token list and how far into it we are; the call stack tracks the nesting
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    // parses the whole file into one tree per expression, collecting every syntax error
    pub fn parse(&mut self) -> (Vec<Expr>, Vec<ParseError>) {
        let mut expressions = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            match self.expression_on_its_own_line() {
                Ok(expr) => expressions.push(expr),
                Err(error) => {
                    errors.push(error);
                    self.skip_rest_of_line();
                }
            }
        }

        (expressions, errors)
    }

    // an expression ends where the grammar says it does, and the next one has to start on a new line
    fn expression_on_its_own_line(&mut self) -> Result<Expr, ParseError> {
        let expr = self.expression()?;
        if !self.is_at_end() && self.peek().line == self.previous().line {
            return Err(self.error(self.peek(), "Expect a new line after expression."));
        }
        Ok(expr)
    }

    // ----- grammar rules, loosest-binding first -----

    // expression → equality
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    // equality → unary ( ( "!=" | "==" ) unary )*
    // comparison, term, and factor will sit between equality and unary once they exist
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_types(&[Tokentypes::NotEqual, Tokentypes::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            // the tree built so far becomes the left operand, so == and != group from the left
            expr = Expr::Binary { left: Box::new(expr), operator, right: Box::new(right) };
        }

        Ok(expr)
    }

    // unary → "!" unary | primary
    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_types(&[Tokentypes::Not]) {
            let operator = self.previous().clone();
            // calling itself is what lets !!nocap nest
            let right = self.unary()?;
            return Ok(Expr::Unary { operator, right: Box::new(right) });
        }
        self.primary()
    }

    // primary → NUM | DEC | STRING | "nocap" | "cap" | "(" expression ")"
    fn primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek().clone();

        let literal = match token.token_type {
            Tokentypes::True => Literal::Boolean(true),
            Tokentypes::False => Literal::Boolean(false),
            Tokentypes::String => Literal::String(token.lexeme.clone()),
            Tokentypes::Num | Tokentypes::Dec => match token.lexeme.parse::<f64>() {
                Ok(value) => Literal::Number(value),
                Err(_) => return Err(self.error(&token, "Invalid number.")),
            },
            Tokentypes::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(Tokentypes::RightParen, "Expect ')' after expression.")?;
                return Ok(Expr::Grouping(Box::new(expr)));
            }
            _ => return Err(self.error(&token, "Expect expression.")),
        };

        self.advance();
        Ok(Expr::Literal(literal))
    }

    // ----- helpers: the token-level versions of the scanner's character helpers -----

    // the current token, not consumed
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    // the token just consumed
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == Tokentypes::Eof
    }

    // consumes the current token; never moves past Eof
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    // is the current token this type? (never true for Eof)
    fn check(&self, token_type: &Tokentypes) -> bool {
        !self.is_at_end() && &self.peek().token_type == token_type
    }

    // consumes the current token only if it is one of these types; this is what a | in the grammar becomes
    fn match_types(&mut self, types: &[Tokentypes]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    // demands a token of this type, which is where "missing )" style errors come from
    fn consume(&mut self, token_type: Tokentypes, message: &str) -> Result<(), ParseError> {
        if self.check(&token_type) {
            self.advance();
            return Ok(());
        }
        Err(self.error(self.peek(), message))
    }

    // after an error, drop the rest of that line so the next line gets parsed on its own.
    // it always consumes at least one token (or stops at Eof), so it can't loop forever
    fn skip_rest_of_line(&mut self) {
        let line = self.peek().line;
        while !self.is_at_end() && self.peek().line == line {
            self.advance();
        }
    }

    fn error(&self, token: &Token, message: &str) -> ParseError {
        let location = if token.token_type == Tokentypes::Eof {
            "end".to_string()
        } else {
            format!("'{}'", token.lexeme)
        };
        ParseError { message: format!("[line {}] Error at {}: {}", token.line, location, message) }
    }
}
