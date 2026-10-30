use crate::ast::{Expr, Literal, Program, Stmt};
use crate::tokens::{LiteralValue, Token, Tokentypes};

// a syntax error, already formatted with its line: [line 1] Error at ')': Expect expression.
pub struct ParseError {
    pub message: String,
}

// recursive descent: one function per grammar rule, each calling the rule below it.
// the cursor tracks the token stream; the call stack tracks expression nesting
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    // program -> declaration* EOF
    // collect every syntax error; return a root only on success
    pub fn parse(&mut self) -> Result<Program, Vec<ParseError>> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            let statement_start = self.current;
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(error) => {
                    errors.push(error);
                    self.synchronize(statement_start);
                }
            }
        }

        if errors.is_empty() {
            Ok(Program { statements })
        } else {
            Err(errors)
        }
    }

    // declaration -> holdDeclaration | statement
    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_types(&[Tokentypes::Hold]) {
            return self.hold_declaration();
        }
        self.statement()
    }

    // holdDeclaration -> "hold" IDENTIFIER ( "=" expression )? ";"
    fn hold_declaration(&mut self) -> Result<Stmt, ParseError> {
        self.consume(Tokentypes::Identifier, "Expect variable name.")?;
        let name = self.previous().clone();

        let initializer = if self.match_types(&[Tokentypes::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(Tokentypes::Semicolon, "Expect ';' after variable declaration.")?;
        Ok(Stmt::Hold { name, initializer })
    }

    // statement -> printStatement | expressionStatement
    // the leading keyword is the only thing that distinguishes them, so one token of
    // lookahead picks the branch and no backtracking is needed
    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_types(&[Tokentypes::Spittin]) {
            return self.print_statement();
        }
        self.expression_statement()
    }

    // printStatement -> "spittin" expression ";"
    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(Tokentypes::Semicolon, "Expect ';' after spittin value.")?;
        Ok(Stmt::Print(expr))
    }

    // expressionStatement -> expression ";"
    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(Tokentypes::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }

    // ----- grammar rules, loosest-binding first -----

    // expression → assignment
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    // assignment → IDENTIFIER "=" assignment | equality
    // the left side parses as an ordinary expression first: one token of lookahead can't tell
    // `a = 1` from `a + 1`. finding "=" afterwards is what checks that it was a bare name
    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.equality()?;

        if self.match_types(&[Tokentypes::Equal]) {
            let equals = self.previous().clone();
            // calling itself is what makes a = b = 1 group from the right
            let value = self.assignment()?;
            return match expr {
                Expr::Variable(name) => Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                }),
                _ => Err(self.error(&equals, "Invalid assignment target.")),
            };
        }

        Ok(expr)
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

    // primary → NUM | DEC | STRING | "nocap" | "cap" | "nil" | IDENTIFIER | "(" expression ")"
    fn primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek().clone();

        let literal = match (&token.token_type, &token.literal) {
            (Tokentypes::Nil, _) => Literal::Nil,
            (Tokentypes::True | Tokentypes::False, Some(LiteralValue::Boolean(value))) => {
                Literal::Boolean(*value)
            }
            (Tokentypes::String, Some(LiteralValue::String(value))) => {
                Literal::String(value.clone())
            }
            (Tokentypes::Num | Tokentypes::Dec, Some(LiteralValue::Number(value))) => {
                Literal::Number(*value)
            }
            (Tokentypes::Identifier, _) => {
                self.advance();
                return Ok(Expr::Variable(token));
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

    // stop after the failed statement's terminator, preserving the next statement.
    // a delimiter already consumed by the failed production is also a recovery boundary
    fn synchronize(&mut self, statement_start: usize) {
        if self.current > statement_start && self.previous().token_type == Tokentypes::Semicolon {
            return;
        }
        while !self.is_at_end() {
            let is_terminator = self.peek().token_type == Tokentypes::Semicolon;
            self.advance();
            if is_terminator {
                return;
            }
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
        }
    }
}
