use crate::ast::{Expr, Literal, Program, Stmt};

// traverse the program root, emitting one formatted top-level statement at a time
pub fn print(program: &Program, mut emit: impl FnMut(String)) {
    for statement in &program.statements {
        emit(print_statement(statement));
    }
}

fn print_statement(statement: &Stmt) -> String {
    match statement {
        Stmt::Expression(expr) => print_expression(expr),
        // the keyword leads, the same way an operator does, so the statement form is visible
        Stmt::Print(expr) => format!("(spittin {})", print_expression(expr)),
        Stmt::Hold {
            name,
            initializer: Some(expr),
        } => format!("(hold {} {})", name.lexeme, print_expression(expr)),
        Stmt::Hold {
            name,
            initializer: None,
        } => format!("(hold {})", name.lexeme),
        // a whole block stays on its own line, its statements in order: (block (hold x 1.0))
        Stmt::Block(statements) => {
            let mut text = String::from("(block");
            for statement in statements {
                text.push(' ');
                text.push_str(&print_statement(statement));
            }
            text.push(')');
            text
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
        Expr::Variable(name) => name.lexeme.clone(),
        Expr::Assign { name, value } => {
            format!("(= {} {})", name.lexeme, print_expression(value))
        }
    }
}
