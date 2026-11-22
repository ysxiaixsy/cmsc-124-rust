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
        Stmt::Block(statements) => print_block(statements),
        // the name, then the parameters in their own parentheses so a motion with none still
        // shows the slot, then the body: (motion add (a b) (block (typeshi (+ a b))))
        Stmt::Motion(declaration) => {
            let params: Vec<&str> = declaration
                .params
                .iter()
                .map(|param| param.lexeme.as_str())
                .collect();
            format!(
                "(motion {} ({}) {})",
                declaration.name.lexeme,
                params.join(" "),
                print_block(&declaration.body)
            )
        }
        Stmt::Return(Some(expr)) => format!("(typeshi {})", print_expression(expr)),
        Stmt::Return(None) => "(typeshi)".to_string(),
        Stmt::Pause => "(pause)".to_string(),
        // the else slot is printed only when it exists, so a bare ong reads (ong cond then)
        Stmt::Ong {
            condition,
            then_branch,
            else_branch,
        } => {
            let mut text = format!(
                "(ong {} {}",
                print_expression(condition),
                print_statement(then_branch)
            );
            if let Some(branch) = else_branch {
                text.push(' ');
                text.push_str(&print_statement(branch));
            }
            text.push(')');
            text
        }
        Stmt::Spin { condition, body } => format!(
            "(spin {} {})",
            print_expression(condition),
            print_statement(body)
        ),
    }
}

// a whole block stays on its own line, its statements in order: (block (hold x 1.0))
fn print_block(statements: &[Stmt]) -> String {
    let mut text = String::from("(block");
    for statement in statements {
        text.push(' ');
        text.push_str(&print_statement(statement));
    }
    text.push(')');
    text
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
        // && and || print the way every other two-operand operator does
        Expr::Binary {
            left,
            operator,
            right,
        }
        | Expr::Logical {
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
        // the callee first, then the arguments in order: (call add 1.0 2.0), and (call f)
        Expr::Call {
            callee, arguments, ..
        } => {
            let mut text = format!("(call {}", print_expression(callee));
            for argument in arguments {
                text.push(' ');
                text.push_str(&print_expression(argument));
            }
            text.push(')');
            text
        }
        Expr::Variable(name) => name.lexeme.clone(),
        Expr::Assign { name, value } => {
            format!("(= {} {})", name.lexeme, print_expression(value))
        }
    }
}
