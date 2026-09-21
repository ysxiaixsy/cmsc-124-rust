use crate::ast::{Expr, Literal};

// prints a tree in prefix form, operator first: (== 1.0 (group (! cap))).
// every operator sits inside its own parentheses, so the grouping can be read straight off the text
pub fn print(expr: &Expr) -> String {
    match expr {
        // {:?} always prints a decimal point, so 5 comes out as 5.0 and every number looks the same
        Expr::Literal(Literal::Number(value)) => format!("{:?}", value),
        // quotes keep a string like "1" from looking like the number 1
        Expr::Literal(Literal::String(text)) => format!("\"{}\"", text),
        Expr::Literal(Literal::Boolean(true)) => "nocap".to_string(),
        Expr::Literal(Literal::Boolean(false)) => "cap".to_string(),
        Expr::Grouping(inner) => format!("(group {})", print(inner)),
        Expr::Unary { operator, right } => format!("({} {})", operator.lexeme, print(right)),
        Expr::Binary { left, operator, right } => {
            format!("({} {} {})", operator.lexeme, print(left), print(right))
        }
    }
}
