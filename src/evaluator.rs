use std::fmt;

use crate::ast::{Expr, Literal, Program, Stmt};
use crate::tokens::{Token, Tokentypes};
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub line: usize,
    pub message: String,
}

impl RuntimeError {
    fn new(token: &Token, message: &str) -> Self {
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

// emit values in source order so a later runtime failure preserves earlier output
pub fn evaluate(program: &Program, mut emit: impl FnMut(Value)) -> Result<(), RuntimeError> {
    for statement in &program.statements {
        match statement {
            Stmt::Expression(expr) => emit(evaluate_expression(expr)?),
        }
    }
    Ok(())
}

// walk the tree directly. week 1 evaluates arithmetic; logical, comparison, and equality
// operators remain valid syntax but report an unsupported-operator error at runtime.
fn evaluate_expression(expr: &Expr) -> Result<Value, RuntimeError> {
    match expr {
        Expr::Literal(literal) => Ok(match literal {
            Literal::Number(value) => Value::Number(*value),
            Literal::String(value) => Value::String(value.clone()),
            Literal::Boolean(value) => Value::Boolean(*value),
            Literal::Nil => Value::Nil,
        }),
        Expr::Grouping(expr) => evaluate_expression(expr),
        Expr::Unary { operator, right } => {
            let right = evaluate_expression(right)?;
            match operator.token_type {
                Tokentypes::Minus => match right {
                    Value::Number(value) => Ok(Value::Number(-value)),
                    _ => Err(RuntimeError::new(operator, "Operand must be a number.")),
                },
                _ => Err(unsupported_operator(operator)),
            }
        }
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            // evaluate in source order and propagate the first operand error
            let left = evaluate_expression(left)?;
            let right = evaluate_expression(right)?;
            match operator.token_type {
                Tokentypes::Plus => match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
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
                    let (Value::Number(left), Value::Number(right)) = (left, right) else {
                        return Err(RuntimeError::new(operator, "Operands must be numbers."));
                    };
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
                _ => Err(unsupported_operator(operator)),
            }
        }
    }
}

fn unsupported_operator(operator: &Token) -> RuntimeError {
    RuntimeError::new(
        operator,
        "Operator is not supported by the Week 1 evaluator.",
    )
}
