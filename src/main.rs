mod tokens;
mod tokenizer;

use std::{env, fs, io::{self, Write}, process};
use tokenizer::{tokenizer, ScanResult};

fn fail(message: &str) -> ! {
    eprintln!("lab 0 error: {}", message);
    process::exit(65);
}



fn main(){
    let args = env::args().collect::<Vec<String>>();

    // no arguments: start the REPL
    if args.len() == 1 {
        repl();
        return;
    }

    if args.len() != 3 || args[1] != "--tokenize" {
        fail("Usage: ./run [--tokenize <source-file>]");
    }

    let filepath = &args[2];

    let contents = fs::read_to_string(filepath).
    unwrap_or_else(|error| fail(&format!("Failed to read file: {}", error)));
    let result = tokenizer(contents);
    if !print_result(result) {
        process::exit(65);
    }
}

// prints the tokens to stdout if there were no errors; otherwise prints the tokens scanned before
// the first error and then every error, all on stderr, since nothing about a rejected file belongs
// on stdout. returns false when there were errors
fn print_result(result: ScanResult) -> bool {
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

// Enter starts a new line and a blank line submits the entry; the session ends when the input closes
fn repl() {
    while let Some(entry) = read_entry() {
        if !entry.is_empty() {
            // an entry with errors prints them and the loop keeps going, unlike file mode
            print_result(tokenizer(entry));
        }
    }
    println!();
}

// reads lines until a blank one and returns them joined; None once the input has closed
fn read_entry() -> Option<String> {
    let mut lines: Vec<String> = Vec::new();
    loop {
        // "> " starts an entry, "... " continues it
        print!("{}", if lines.is_empty() { "> " } else { "... " });
        io::stdout().flush().unwrap(); // print! doesn't flush on its own, so the prompt would stay hidden

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            // 0 bytes means the input closed (Ctrl+D): scan what was typed, or stop if nothing was
            Ok(0) => return if lines.is_empty() { None } else { Some(lines.join("\n")) },
            Ok(_) => {}
            Err(error) => {
                eprintln!("Error: Failed to read input: {}", error);
                return None;
            }
        }

        let line = line.trim_end();
        if line.is_empty() {
            // joined without a trailing newline, so Eof lands on the last typed line
            return Some(lines.join("\n"));
        }
        lines.push(line.to_string());
    }
}