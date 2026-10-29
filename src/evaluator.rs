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

// emit values in source order so a later runtime failure preserves earlier output
pub fn evaluate(
    program: &Program,
    output: Output,
    mut emit: impl FnMut(Value),
) -> Result<(), RuntimeError> {
    for statement in &program.statements {
        match statement {
            // evaluated either way: the value is what gets dropped, never the work
            Stmt::Expression(expr) => {
                let value = evaluate_expression(expr)?;
                if let Output::EveryValue = output {
                    emit(value);
                }
            }
            Stmt::Print(expr) => emit(evaluate_expression(expr)?),
        }
    }
    Ok(())
}

// walk the tree directly: evaluate the children first, then apply the operator to their values
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
                // ! takes any value on purpose: the truthiness rule decides, so there is
                // nothing to type check
                Tokentypes::Not => Ok(Value::Boolean(!is_truthy(&right))),
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
                // comparison is numbers only, and answers with a boolean rather than a number
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
                // equality takes any pair. Value compares its variant first, so a number and
                // a string are unequal rather than an error, and nil equals only nil
                Tokentypes::EqualEqual => Ok(Value::Boolean(left == right)),
                Tokentypes::NotEqual => Ok(Value::Boolean(left != right)),
                _ => Err(unsupported_operator(operator)),
            }
        }
    }
}

// only cap and nil are falsey. zero and the empty string are truthy, and writing the rule
// once is what stops ! and any later conditional from disagreeing about it
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
