mod ast;
mod evaluator;
mod parser;
mod printer;
mod tokenizer;
mod tokens;
mod value;

use parser::Parser;
use std::{
    env, fs,
    io::{self, Write},
    path::Path,
    process::ExitCode,
};
use tokenizer::tokenizer;

const USAGE: &str = "Usage:\n  ./run\n  ./run --tokenize\n  ./run --parse\n  ./run <source-file>\n  ./run --tokenize <source-file>\n  ./run --parse <source-file>\n  ./run --eval <source-file>\n  ./run --eval\n  ./run --help";

#[derive(Clone, Copy)]
enum Stage {
    Run, // Lab 0: echo the source until program execution arrives in Lab 4.
    Tokenize,
    Parse,
    Eval,
}

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(first) = args.next() else {
        repl(Stage::Eval);
        return ExitCode::SUCCESS;
    };

    let (stage, path) = match first.to_str() {
        Some("--help") => {
            if args.next().is_some() {
                return usage_error("--help does not take a source file");
            }
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Some("--tokenize") => (Stage::Tokenize, args.next()),
        Some("--parse") => (Stage::Parse, args.next()),
        Some("--eval") => (Stage::Eval, args.next()),
        _ if first.to_string_lossy().starts_with('-') => {
            return usage_error(&format!("Unknown flag: {}", first.to_string_lossy()));
        }
        _ => (Stage::Run, Some(first)),
    };
    if args.next().is_some() {
        return usage_error("Too many arguments: expected one source file");
    }

    if let Some(path) = path {
        match fs::read_to_string(&path) {
            Ok(source) => ExitCode::from(run_source(stage, source)),
            Err(error) => {
                eprintln!(
                    "Error: Failed to read '{}': {error}",
                    Path::new(&path).display()
                );
                ExitCode::from(65)
            }
        }
    } else {
        repl(stage);
        ExitCode::SUCCESS
    }
}

fn usage_error(message: &str) -> ExitCode {
    eprintln!("Error: {message}\n{USAGE}");
    ExitCode::from(65)
}

// Files and REPL lines share the same scan, parse, and evaluation path.
fn run_source(stage: Stage, source: String) -> u8 {
    if matches!(stage, Stage::Run) {
        print!("{source}");
        return 0;
    }

    let scanned = tokenizer(source);
    if !scanned.errors.is_empty() {
        // Token mode also reports the tokens preceding the first scan error.
        if let Stage::Tokenize = stage {
            if let Some(end) = scanned.first_error_at {
                for token in &scanned.tokens[..end] {
                    eprintln!("{token:#?}");
                }
            }
        }
        for error in scanned.errors {
            eprintln!("Error: {error}");
        }
        return 65;
    }
    if let Stage::Tokenize = stage {
        for token in scanned.tokens {
            println!("{token:#?}");
        }
        return 0;
    }

    // Reject the whole source before printing trees or evaluating expressions.
    let program = match Parser::new(scanned.tokens).parse() {
        Ok(program) => program,
        Err(errors) => {
            for error in errors {
                eprintln!("{}", error.message);
            }
            return 65;
        }
    };
    match stage {
        Stage::Parse => printer::print(&program, |tree| println!("{tree}")),
        Stage::Eval => {
            if let Err(error) = evaluator::evaluate(&program, |value| println!("{value}")) {
                eprintln!("{error}");
                return 70;
            }
        }
        Stage::Run | Stage::Tokenize => unreachable!(),
    }
    0
}

fn repl(stage: Stage) {
    loop {
        print!("> ");
        if let Err(error) = io::stdout().flush() {
            eprintln!("Error: Failed to write prompt: {error}");
            break;
        }

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                // Trim only the line ending; spaces may belong to a string.
                run_source(stage, line.trim_end_matches(['\r', '\n']).to_string());
            }
            Err(error) => {
                eprintln!("Error: Failed to read input: {error}");
                break;
            }
        }
    }
    println!();
}
