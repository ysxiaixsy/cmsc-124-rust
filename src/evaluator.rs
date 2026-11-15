use std::fmt;
use std::rc::Rc;

use crate::ast::{Expr, Literal, Program, Stmt};
use crate::environment::Environment;
use crate::tokens::{Token, Tokentypes};
use crate::value::{Function, Value};

// how much of the interpreter's stack calls may fill before the next one is refused. every
// call runs on the host stack, so a motion that never stops calling itself would otherwise
// overflow it and abort the process. the quarter held back is for the last call's own body,
// which the parser has already limited to MAX_NESTING levels
const CALL_STACK_BYTES: usize = crate::STACK_BYTES / 4 * 3;

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub line: usize,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: &Token, message: &str) -> Self {
        Self {
            line: token.line,
            message: message.to_string(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}] Runtime error: {}", self.line, self.message)
    }
}

impl std::error::Error for RuntimeError {}

// what a bare expression statement does with its value. a `spittin` statement always emits;
// this only decides whether an expression statement does too, which is the single difference
// between running a program and inspecting one with --eval
#[derive(Clone, Copy)]
pub enum Output {
    // --eval, the Lab 3 contract: every expression statement shows its value
    EveryValue,
    // plain ./run and the Lab 4 contract: a program speaks only through `spittin`
    PrintOnly,
}

// how a statement finished. a typeshi has to end every block and loop between itself and its
// call, so each of them hands this back up instead of carrying on
enum Flow {
    // ran to its end: the next statement goes ahead
    Next,
    // a typeshi ran: stop, and give this value to the call
    Return(Value),
}

// emit values in source order so a later runtime failure preserves earlier output.
// the caller owns the environment, so bindings can outlive one call: a REPL line, in practice
pub fn evaluate(
    program: &Program,
    environment: &Environment,
    output: Output,
    mut emit: impl FnMut(Value),
) -> Result<(), RuntimeError> {
    let mut interpreter = Interpreter {
        output,
        emit: &mut emit,
        stack_base: stack_address(),
    };
    // the parser rejects a typeshi outside a motion, so nothing at the top level can return
    interpreter.execute_all(&program.statements, environment)?;
    Ok(())
}

// roughly where the host stack has reached: the address of a local in the current frame
fn stack_address() -> usize {
    let marker = 0u8;
    std::hint::black_box(&marker) as *const u8 as usize
}

// what stays the same for a whole run, so the tree walk only has to pass the scope along
struct Interpreter<'a> {
    output: Output,
    emit: &'a mut dyn FnMut(Value),
    // where the host stack stood when the run began. calls measure their depth from here
    stack_base: usize,
}

impl Interpreter<'_> {
    // statements run in source order. the first runtime error stops the rest, and so does the
    // first typeshi
    fn execute_all(
        &mut self,
        statements: &[Stmt],
        environment: &Environment,
    ) -> Result<Flow, RuntimeError> {
        for statement in statements {
            if let Flow::Return(value) = self.execute(statement, environment)? {
                return Ok(Flow::Return(value));
            }
        }
        Ok(Flow::Next)
    }

    fn execute(
        &mut self,
        statement: &Stmt,
        environment: &Environment,
    ) -> Result<Flow, RuntimeError> {
        match statement {
            // evaluated either way: the value is what gets dropped, never the work
            Stmt::Expression(expr) => {
                let value = self.evaluate_expression(expr, environment)?;
                if let Output::EveryValue = self.output {
                    (self.emit)(value);
                }
            }
            Stmt::Print(expr) => {
                let value = self.evaluate_expression(expr, environment)?;
                (self.emit)(value);
            }
            Stmt::Hold { name, initializer } => {
                // the name is bound only after the initializer finishes, so `hold x = x + 1;`
                // reads the previous x, even when that x lives in an enclosing scope and this
                // hold is about to shadow it. `hold y = y;` fails unless some scope has a y
                let value = match initializer {
                    Some(expr) => self.evaluate_expression(expr, environment)?,
                    None => Value::Nil,
                };
                environment.define(&name.lexeme, value);
            }
            // a declaration is a hold whose value is a function. the function keeps the scope
            // it was declared in, and that scope now holds the function's own name, which is
            // all recursion needs
            Stmt::Motion(declaration) => {
                let function = Function {
                    declaration: Rc::clone(declaration),
                    closure: environment.clone(),
                };
                environment.define(
                    &declaration.name.lexeme,
                    Value::Function(Rc::new(function)),
                );
            }
            Stmt::Return(value) => {
                let value = match value {
                    Some(expr) => self.evaluate_expression(expr, environment)?,
                    None => Value::Nil,
                };
                return Ok(Flow::Return(value));
            }
            // only the chosen branch is executed. the other one is never walked, so a runtime
            // error sitting in it cannot fire
            Stmt::Ong {
                condition,
                then_branch,
                else_branch,
            } => {
                if is_truthy(&self.evaluate_expression(condition, environment)?) {
                    return self.execute(then_branch, environment);
                } else if let Some(branch) = else_branch {
                    return self.execute(branch, environment);
                }
            }
            // the condition is re-evaluated before every pass, so a body that changes what the
            // condition reads is what eventually stops the loop. a typeshi in the body stops
            // it too, on its way out of the call
            Stmt::Spin { condition, body } => {
                while is_truthy(&self.evaluate_expression(condition, environment)?) {
                    if let Flow::Return(value) = self.execute(body, environment)? {
                        return Ok(Flow::Return(value));
                    }
                }
            }
            // the block's scope is released when this arm ends, however it ends, unless a
            // function declared inside it is still holding on. the enclosing scope was never
            // replaced, so there is nothing to restore after an error
            Stmt::Block(statements) => {
                return self.execute_all(statements, &environment.child());
            }
        }
        Ok(Flow::Next)
    }

    // walk the tree directly: evaluate the children first, then apply the operator to their
    // values
    fn evaluate_expression(
        &mut self,
        expr: &Expr,
        environment: &Environment,
    ) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(literal) => Ok(match literal {
                Literal::Number(value) => Value::Number(*value),
                Literal::String(value) => Value::String(value.clone()),
                Literal::Boolean(value) => Value::Boolean(*value),
                Literal::Nil => Value::Nil,
            }),
            Expr::Grouping(expr) => self.evaluate_expression(expr, environment),
            Expr::Variable(name) => environment.get(name),
            // the right side runs first, and the assignment's own value is what it produces,
            // which is why `spittin x = 1;` and `a = b = 1;` work
            Expr::Assign { name, value } => {
                let value = self.evaluate_expression(value, environment)?;
                environment.assign(name, value.clone())?;
                Ok(value)
            }
            Expr::Unary { operator, right } => {
                let right = self.evaluate_expression(right, environment)?;
                match operator.token_type {
                    Tokentypes::Minus => match right {
                        Value::Number(value) => Ok(Value::Number(-value)),
                        _ => Err(RuntimeError::new(operator, "Operand must be a number.")),
                    },
                    // ! takes any value on purpose: the truthiness rule decides, so there is
                    // nothing to type check
                    Tokentypes::Not => Ok(Value::Boolean(!is_truthy(&right))),
                    _ => Err(unsupported_operator(operator)),
                }
            }
            // the left operand settles the answer whenever it can, and then the right one is
            // never evaluated: no side effects, no runtime errors. the result is whichever
            // operand decided, itself, not a boolean made from it
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate_expression(left, environment)?;
                let settled = match operator.token_type {
                    Tokentypes::Or => is_truthy(&left),
                    Tokentypes::And => !is_truthy(&left),
                    _ => return Err(unsupported_operator(operator)),
                };
                if settled {
                    Ok(left)
                } else {
                    self.evaluate_expression(right, environment)
                }
            }
            // the callee, then each argument left to right, then the call. the callee is
            // checked only after the arguments have run, so an error inside an argument is
            // the one reported
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate_expression(callee, environment)?;
                let mut values = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    values.push(self.evaluate_expression(argument, environment)?);
                }
                match callee {
                    Value::Function(function) => self.call(&function, values, paren),
                    _ => Err(RuntimeError::new(paren, "Can only call motions.")),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                // evaluate in source order and propagate the first operand error
                let left = self.evaluate_expression(left, environment)?;
                let right = self.evaluate_expression(right, environment)?;
                match operator.token_type {
                    Tokentypes::Plus => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left + right))
                        }
                        (Value::String(mut left), Value::String(right)) => {
                            left.push_str(&right);
                            Ok(Value::String(left))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be two numbers or two strings.",
                        )),
                    },
                    Tokentypes::Minus | Tokentypes::Star | Tokentypes::Slash => {
                        let (left, right) = number_operands(operator, &left, &right)?;
                        let value = match operator.token_type {
                            Tokentypes::Minus => left - right,
                            Tokentypes::Star => left * right,
                            Tokentypes::Slash if right == 0.0 => {
                                return Err(RuntimeError::new(operator, "Division by zero."));
                            }
                            Tokentypes::Slash => left / right,
                            _ => unreachable!(),
                        };
                        Ok(Value::Number(value))
                    }
                    // comparison is numbers only, and answers with a boolean rather than a
                    // number
                    Tokentypes::LessThan
                    | Tokentypes::LessThanEqual
                    | Tokentypes::GreaterThan
                    | Tokentypes::GreaterThanEqual => {
                        let (left, right) = number_operands(operator, &left, &right)?;
                        let value = match operator.token_type {
                            Tokentypes::LessThan => left < right,
                            Tokentypes::LessThanEqual => left <= right,
                            Tokentypes::GreaterThan => left > right,
                            Tokentypes::GreaterThanEqual => left >= right,
                            _ => unreachable!(),
                        };
                        Ok(Value::Boolean(value))
                    }
                    // equality takes any pair. Value compares its variant first, so a number
                    // and a string are unequal rather than an error, and nil equals only nil
                    Tokentypes::EqualEqual => Ok(Value::Boolean(left == right)),
                    Tokentypes::NotEqual => Ok(Value::Boolean(left != right)),
                    _ => Err(unsupported_operator(operator)),
                }
            }
        }
    }

    // run a function's body in a new scope that holds its parameters. that scope sits inside
    // the one the function was declared in, not the caller's, so the body reads names from
    // where it was written
    fn call(
        &mut self,
        function: &Function,
        arguments: Vec<Value>,
        paren: &Token,
    ) -> Result<Value, RuntimeError> {
        let declaration = &function.declaration;
        if arguments.len() != declaration.params.len() {
            let message = format!(
                "Expected {} arguments but got {}.",
                declaration.params.len(),
                arguments.len()
            );
            return Err(RuntimeError::new(paren, &message));
        }
        if self.stack_base.abs_diff(stack_address()) > CALL_STACK_BYTES {
            return Err(RuntimeError::new(paren, "Stack overflow."));
        }

        let scope = function.closure.child();
        for (param, argument) in declaration.params.iter().zip(arguments) {
            scope.define(&param.lexeme, argument);
        }

        // running off the end of the body is the same as a bare `typeshi;`
        match self.execute_all(&declaration.body, &scope)? {
            Flow::Return(value) => Ok(value),
            Flow::Next => Ok(Value::Nil),
        }
    }
}

// only cap and nil are falsey. zero and the empty string are truthy, and writing the rule
// once is what stops !, the conditionals, and the logical operators from disagreeing about it
fn is_truthy(value: &Value) -> bool {
    !matches!(value, Value::Boolean(false) | Value::Nil)
}

// every operator that needs two numbers asks here, so their diagnostics cannot drift apart
fn number_operands(
    operator: &Token,
    left: &Value,
    right: &Value,
) -> Result<(f64, f64), RuntimeError> {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => Ok((*left, *right)),
        _ => Err(RuntimeError::new(operator, "Operands must be numbers.")),
    }
}

// the parser never builds any other operator into a unary or binary node. this reports a
// diagnostic instead of panicking if the grammar ever grows one the evaluator has not met
fn unsupported_operator(operator: &Token) -> RuntimeError {
    RuntimeError::new(operator, "Unsupported operator.")
}
