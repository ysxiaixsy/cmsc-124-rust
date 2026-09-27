mod ast;
mod parser;
mod printer;
mod tokens;
mod tokenizer;

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

const USAGE: &str = "Usage:\n  ./run\n  ./run <source-file>\n  ./run --tokenize <source-file>\n  ./run --parse <source-file>\n  ./run --help";

enum Stage {
    // plain ./run <file>. there is no interpreter until Lab 4, so for now it prints the file back
    Run,
    Tokenize,
    Parse,
}

enum Command {
    Repl,
    File { stage: Stage, path: PathBuf },
    Help,
}

fn parse_command(mut args: impl Iterator<Item = OsString>) -> Result<Command, String> {
    let Some(flag) = args.next() else {
        return Ok(Command::Repl);
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
    } else if flag.to_string_lossy().starts_with('-') {
        return Err(format!("Unknown flag: {}", flag.to_string_lossy()));
    } else {
        (Stage::Run, Some(flag.clone()))
    };

    let path = path.ok_or_else(|| format!("Missing source file after {}", flag.to_string_lossy()))?;
    if args.next().is_some() {
        return Err("Too many arguments: expected one source file".into());
    }

    Ok(Command::File { stage, path: path.into() })
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
        Command::Repl => {
            repl();
            ExitCode::SUCCESS
        }
        Command::Help => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Command::File { stage, path } => match run_file(stage, &path) {
            Ok(true) => ExitCode::SUCCESS,
            Ok(false) => ExitCode::from(65),
            Err(error) => {
                eprintln!("Error: Failed to read '{}': {error}", path.display());
                ExitCode::from(65)
            }
        },
    }
}

fn run_file(stage: Stage, path: &Path) -> io::Result<bool> {
    let source = fs::read_to_string(path)?;
    Ok(match stage {
        Stage::Run => {
            print!("{source}");
            true
        }
        Stage::Tokenize => report_tokens(tokenizer(source)),
        Stage::Parse => report_parse(tokenizer(source)),
    })
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

// each submitted line is scanned on its own; an error does not end the session
fn repl() {
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
                report_tokens(tokenizer(source.to_string()));
            }
            Err(error) => {
                eprintln!("Error: Failed to read input: {error}");
                break;
            }
        }
    }
    println!();
}
