use crate::ast::{Expr, Literal};
use crate::tokens::{LiteralValue, Token, Tokentypes};

// a syntax error, already formatted with its line: [line 1] Error at ')': Expect expression.
pub struct ParseError {
    pub message: String,
    recovery_line: usize,
}

// recursive descent: one function per grammar rule, each calling the rule below it.
// the cursor and starting line track the expression; the call stack tracks its nesting
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    // the line the expression being parsed started on; tokens on later lines belong to the next one
    line: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
            line: 1,
        }
    }

    // parses the whole file into one tree per expression, collecting every syntax error
    pub fn parse(&mut self) -> (Vec<Expr>, Vec<ParseError>) {
        let mut expressions = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            match self.expression_on_its_own_line() {
                Ok(expr) => expressions.push(expr),
                Err(error) => {
                    let recovery_line = error.recovery_line;
                    errors.push(error);
                    self.skip_rest_of_line(recovery_line);
                }
            }
        }

        (expressions, errors)
    }

    // each expression starts on its own line. a string may span lines, but nothing else may
    // follow the expression on its final line
    fn expression_on_its_own_line(&mut self) -> Result<Expr, ParseError> {
        self.line = self.peek().line;
        let expr = self.expression()?;
        let end_line = self.previous().line + self.previous().lexeme.matches('\n').count();
        if !self.is_at_end() && self.peek().line <= end_line {
            let mut error = self.error(self.peek(), "Expect a new line after expression.");
            error.recovery_line = end_line;
            return Err(error);
        }
        Ok(expr)
    }

    // ----- grammar rules, loosest-binding first -----

    // expression → equality
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    // equality → comparison ( ( "!=" | "==" ) comparison )*
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_types(&[Tokentypes::NotEqual, Tokentypes::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            // the tree built so far becomes the left operand, so == and != group from the left
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )*
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.match_types(&[
            Tokentypes::GreaterThan,
            Tokentypes::GreaterThanEqual,
            Tokentypes::LessThan,
            Tokentypes::LessThanEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // term → factor ( ( "-" | "+" ) factor )*
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_types(&[Tokentypes::Minus, Tokentypes::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // factor → unary ( ( "/" | "*" ) unary )*
    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_types(&[Tokentypes::Slash, Tokentypes::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // unary → ( "!" | "-" ) unary | primary
    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_types(&[Tokentypes::Not, Tokentypes::Minus]) {
            let operator = self.previous().clone();
            // calling itself is what lets !!nocap and --10 nest
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    // primary → NUM | DEC | STRING | "nocap" | "cap" | "(" expression ")"
    fn primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek().clone();

        // an operand on a later line belongs to the next expression, so the one here is unfinished
        if token.line != self.line {
            return Err(self.error_at_line_end("Expect expression."));
        }

        let literal = match (&token.token_type, &token.literal) {
            (Tokentypes::True | Tokentypes::False, Some(LiteralValue::Boolean(value))) => {
                Literal::Boolean(*value)
            }
            (Tokentypes::String, Some(LiteralValue::String(value))) => {
                Literal::String(value.clone())
            }
            (Tokentypes::Num | Tokentypes::Dec, Some(LiteralValue::Number(value))) => {
                Literal::Number(*value)
            }
            (Tokentypes::LeftParen, _) => {
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

    // is the current token this type? (never true for Eof, or for a token on a later line)
    fn check(&self, token_type: &Tokentypes) -> bool {
        !self.is_at_end() && self.peek().line == self.line && &self.peek().token_type == token_type
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
        if self.peek().line != self.line {
            return Err(self.error_at_line_end(message));
        }
        Err(self.error(self.peek(), message))
    }

    // after an error, drop only tokens on the failed expression's final line. if the next token
    // starts a fresh line, leave it for the next parse attempt
    fn skip_rest_of_line(&mut self, line: usize) {
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
        ParseError {
            message: format!("[line {}] Error at {}: {}", token.line, location, message),
            recovery_line: self.line,
        }
    }

    fn error_at_line_end(&self, message: &str) -> ParseError {
        ParseError {
            message: format!("[line {}] Error at end: {}", self.line, message),
            recovery_line: self.line,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::{printer, tokenizer::tokenizer};

    #[test]
    fn recovers_each_incomplete_line_and_keeps_the_final_expression() {
        let source = "1 +\n2 +\n3 +\n4\n";
        let (expressions, errors) = Parser::new(tokenizer(source.to_string()).tokens).parse();

        assert_eq!(errors.len(), 3);
        assert_eq!(expressions.len(), 1);
        assert_eq!(printer::print(&expressions[0]), "4.0");
    }

    #[test]
    fn multiline_string_occupies_its_closing_line() {
        let source = "\"first\nsecond\" 7\n8\n";
        let (expressions, errors) = Parser::new(tokenizer(source.to_string()).tokens).parse();

        assert_eq!(errors.len(), 1);
        assert_eq!(expressions.len(), 1);
        assert_eq!(printer::print(&expressions[0]), "8.0");
    }
}
