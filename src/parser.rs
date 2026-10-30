use crate::ast::{Expr, Literal, Program, Stmt};
use crate::tokens::{LiteralValue, Token, Tokentypes};

// the deepest a statement's tree can get, counting each enclosing block and each expression
// level. the parser, the evaluator, the printer, and dropping the tree all recurse once per
// level, so this has to fit inside main::STACK_BYTES. past it the parser reports a syntax
// error, because overflowing the stack aborts the process
const MAX_NESTING: usize = 10_000;

// a syntax error, already formatted with its line: [line 1] Error at ')': Expect expression.
pub struct ParseError {
    pub message: String,
}

// recursive descent: one function per grammar rule, each calling the rule below it.
// the cursor tracks the token stream; the call stack tracks expression nesting
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    // how deep the tree of the current statement reaches so far: the levels above the rule
    // that is running, plus the height of what that rule has already built
    depth: usize,
    // how many blocks enclose the statement being parsed. its expressions start this deep
    block_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
            depth: 0,
            block_depth: 0,
        }
    }

    // program -> declaration* EOF
    // collect every syntax error; return a root only on success
    pub fn parse(&mut self) -> Result<Program, Vec<ParseError>> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            let statement_start = self.current;
            // an error inside a block unwinds straight to here, past every block it was in
            self.block_depth = 0;
            self.depth = 0;
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

    // statement -> block | printStatement | expressionStatement
    // the leading token is the only thing that distinguishes them, so one token of
    // lookahead picks the branch and no backtracking is needed
    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_types(&[Tokentypes::LeftBrace]) {
            return self.block();
        }
        if self.match_types(&[Tokentypes::Spittin]) {
            return self.print_statement();
        }
        self.expression_statement()
    }

    // block -> "{" declaration* "}"
    // a block is a statement, so it nests anywhere a statement can, and its closing brace
    // ends it: no semicolon follows
    fn block(&mut self) -> Result<Stmt, ParseError> {
        let brace = self.previous().clone();
        if self.block_depth >= MAX_NESTING {
            let message = format!("Blocks nested too deeply (limit is {MAX_NESTING} levels).");
            return Err(self.error(&brace, &message));
        }
        self.block_depth += 1;

        let mut statements = Vec::new();
        while !self.check(&Tokentypes::RightBrace) && !self.is_at_end() {
            self.depth = self.block_depth;
            statements.push(self.declaration()?);
        }

        self.block_depth -= 1;
        self.consume(Tokentypes::RightBrace, "Expect '}' after block.")?;
        Ok(Stmt::Block(statements))
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
            self.nest(&equals)?;
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
        let start = self.depth;
        let mut expr = self.comparison()?;

        while self.match_types(&[Tokentypes::NotEqual, Tokentypes::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.right_operand(start, &operator, Self::comparison)?;
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
        let start = self.depth;
        let mut expr = self.term()?;

        while self.match_types(&[
            Tokentypes::GreaterThan,
            Tokentypes::GreaterThanEqual,
            Tokentypes::LessThan,
            Tokentypes::LessThanEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.right_operand(start, &operator, Self::term)?;
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
        let start = self.depth;
        let mut expr = self.factor()?;

        while self.match_types(&[Tokentypes::Minus, Tokentypes::Plus]) {
            let operator = self.previous().clone();
            let right = self.right_operand(start, &operator, Self::factor)?;
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
        let start = self.depth;
        let mut expr = self.unary()?;

        while self.match_types(&[Tokentypes::Slash, Tokentypes::Star]) {
            let operator = self.previous().clone();
            let right = self.right_operand(start, &operator, Self::unary)?;
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
            self.nest(&operator)?;
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
                self.nest(&token)?;
                let expr = self.expression()?;
                self.consume(Tokentypes::RightParen, "Expect ')' after expression.")?;
                return Ok(Expr::Grouping(Box::new(expr)));
            }
            _ => return Err(self.error(&token, "Expect expression.")),
        };

        self.advance();
        Ok(Expr::Literal(literal))
    }

    // ----- nesting: keeps the tree's depth inside MAX_NESTING -----

    // what a binary rule calls for each right operand. the loops never recurse on the left
    // spine, but each operator still sits on top of everything built so far, so 1 + 1 + 1 is a
    // deep tree. the operand starts back at this rule's own level, one below its operator, and
    // whichever side reached further decides the depth. that way (a) + (b) counts as the deeper
    // of the two, while (a + b) + c keeps growing what the group already built
    fn right_operand(
        &mut self,
        start: usize,
        operator: &Token,
        operand: fn(&mut Self) -> Result<Expr, ParseError>,
    ) -> Result<Expr, ParseError> {
        let left_depth = self.depth;
        self.depth = start;
        self.nest(operator)?;
        let right = operand(self)?;
        self.depth = self.depth.max(left_depth + 1);
        self.check_nesting(operator)?;
        Ok(right)
    }

    // one level deeper, for anything that recurses: a group, a unary operand, or an assignment
    fn nest(&mut self, token: &Token) -> Result<(), ParseError> {
        self.depth += 1;
        self.check_nesting(token)
    }

    fn check_nesting(&self, token: &Token) -> Result<(), ParseError> {
        if self.depth > MAX_NESTING {
            let message = format!("Expression nested too deeply (limit is {MAX_NESTING} levels).");
            return Err(self.error(token, &message));
        }
        Ok(())
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
