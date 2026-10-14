use crate::ast::{Expr, Literal, Program, Stmt};

// traverse the program root, emitting one formatted expression statement at a time
pub fn print(program: &Program, mut emit: impl FnMut(String)) {
    for statement in &program.statements {
        match statement {
            Stmt::Expression(expr) => emit(print_expression(expr)),
        }
    }
}

// prints a tree in prefix form, operator first: (== 1.0 (group (! cap))).
// every operator sits inside its own parentheses, so the grouping can be read straight off the text
fn print_expression(expr: &Expr) -> String {
    match expr {
        Expr::Literal(Literal::Nil) => "nil".to_string(),
        // {:?} always prints a decimal point, so 5 comes out as 5.0 and every number looks the same
        Expr::Literal(Literal::Number(value)) => format!("{:?}", value),
        // debug escaping keeps multiline strings on one output line and backslashes unambiguous
        Expr::Literal(Literal::String(text)) => format!("{text:?}"),
        Expr::Literal(Literal::Boolean(true)) => "nocap".to_string(),
        Expr::Literal(Literal::Boolean(false)) => "cap".to_string(),
        Expr::Grouping(inner) => format!("(group {})", print_expression(inner)),
        Expr::Unary { operator, right } => {
            format!("({} {})", operator.lexeme, print_expression(right))
        }
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            format!(
                "({} {} {})",
                operator.lexeme,
                print_expression(left),
                print_expression(right)
            )
        }
    }
}
