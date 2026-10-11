mod ast;
mod evaluator;
mod parser;
mod printer;
mod tokenizer;
mod tokens;
mod value;

use parser::Parser;
use std::{
    env,
    ffi::{OsStr, OsString},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};
use tokenizer::{tokenizer, ScanResult};

const USAGE: &str = "Usage:\n  ./run\n  ./run --tokenize\n  ./run --parse\n  ./run <source-file>\n  ./run --tokenize <source-file>\n  ./run --parse <source-file>\n  ./run --eval <source-file>\n  ./run --eval\n  ./run --help";

#[derive(Clone, Copy)]
enum Stage {
    // plain ./run <file>. there is no interpreter until Lab 4, so for now it prints the file back
    Run,
    Tokenize,
    Parse,
    Eval,
}

enum Command {
    // each REPL session runs one stage; evaluation is the default
    Repl(Stage),
    File { stage: Stage, path: PathBuf },
    Help,
}

fn parse_command(mut args: impl Iterator<Item = OsString>) -> Result<Command, String> {
    let Some(flag) = args.next() else {
        // Lab 3 makes the no-argument REPL evaluate expressions
        return Ok(Command::Repl(Stage::Eval));
    };

    if flag == OsStr::new("--help") {
        return if args.next().is_none() {
            Ok(Command::Help)
        } else {
            Err("--help does not take a source file".into())
        };
    }

    // with no flag, the first argument is already the source file
    let (stage, path) = if flag == OsStr::new("--tokenize") {
        (Stage::Tokenize, args.next())
    } else if flag == OsStr::new("--parse") {
        (Stage::Parse, args.next())
    } else if flag == OsStr::new("--eval") {
        (Stage::Eval, args.next())
    } else if flag.to_string_lossy().starts_with('-') {
        return Err(format!("Unknown flag: {}", flag.to_string_lossy()));
    } else {
        (Stage::Run, Some(flag.clone()))
    };

    // a stage flag with no source file starts the REPL at that stage
    let Some(path) = path else {
        return Ok(Command::Repl(stage));
    };
    if args.next().is_some() {
        return Err("Too many arguments: expected one source file".into());
    }

    Ok(Command::File {
        stage,
        path: path.into(),
    })
}

fn main() -> ExitCode {
    let command = match parse_command(env::args_os().skip(1)) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("Error: {message}\n{USAGE}");
            return ExitCode::from(65);
        }
    };

    match command {
        Command::Repl(stage) => {
            repl(stage);
            ExitCode::SUCCESS
        }
        Command::Help => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Command::File { stage, path } => match run_file(stage, &path) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                eprintln!("Error: Failed to read '{}': {error}", path.display());
                ExitCode::from(65)
            }
        },
    }
}

fn run_file(stage: Stage, path: &Path) -> io::Result<u8> {
    let source = fs::read_to_string(path)?;
    Ok(match stage {
        Stage::Run => {
            print!("{source}");
            0
        }
        Stage::Tokenize => {
            if report_tokens(tokenizer(source)) {
                0
            } else {
                65
            }
        }
        Stage::Parse => {
            if report_parse(tokenizer(source)) {
                0
            } else {
                65
            }
        }
        Stage::Eval => report_eval(tokenizer(source)),
    })
}

// Validate the whole file before executing any expression. Runtime errors stop only evaluation.
fn report_eval(result: ScanResult) -> u8 {
    if !result.errors.is_empty() {
        for error in result.errors {
            eprintln!("Error: {error}");
        }
        return 65;
    }
    let (expressions, errors) = Parser::new(result.tokens).parse();
    if !errors.is_empty() {
        for error in errors {
            eprintln!("{}", error.message);
        }
        return 65;
    }
    for expression in &expressions {
        match evaluator::evaluate(expression) {
            Ok(value) => println!("{value}"),
            Err(error) => {
                eprintln!("{error}");
                return 70;
            }
        }
    }
    0
}

// parses the scanned tokens and prints one tree per expression to stdout. scan errors and syntax
// errors go to stderr instead, with nothing on stdout. returns false when there were errors
fn report_parse(result: ScanResult) -> bool {
    // the parser only runs on a clean scan; there's no point parsing tokens around a bad character
    if !result.errors.is_empty() {
        for error in result.errors {
            eprintln!("Error: {}", error);
        }
        return false;
    }

    let (expressions, errors) = Parser::new(result.tokens).parse();
    if !errors.is_empty() {
        for error in errors {
            eprintln!("{}", error.message);
        }
        return false;
    }

    for expr in &expressions {
        println!("{}", printer::print(expr));
    }
    true
}

// prints the tokens to stdout if there were no errors; otherwise prints the tokens scanned before
// the first error and then every error, all on stderr, since nothing about a rejected file belongs
// on stdout. returns false when there were errors
fn report_tokens(result: ScanResult) -> bool {
    if let Some(first_error_at) = result.first_error_at {
        for token in &result.tokens[..first_error_at] {
            eprintln!("{:#?}", token);
        }
        for error in result.errors {
            eprintln!("Error: {}", error);
        }
        return false;
    }
    for token in result.tokens {
        println!("{:#?}", token);
    }
    true
}

// each submitted line goes through the stage on its own; an error does not end the session
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
                // remove the line ending only; other trailing spaces may belong to a string
                let source = line.trim_end_matches(['\r', '\n']);
                match stage {
                    Stage::Eval => {
                        report_eval(tokenizer(source.to_string()));
                        true
                    }
                    Stage::Parse => report_parse(tokenizer(source.to_string())),
                    // Run never reaches the REPL: it always comes with a source file
                    Stage::Tokenize | Stage::Run => report_tokens(tokenizer(source.to_string())),
                };
            }
            Err(error) => {
                eprintln!("Error: Failed to read input: {error}");
                break;
            }
        }
    }
    println!();
}
