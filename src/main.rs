use std::{env, fs, process};


fn fail(message: &str) -> ! {
    eprintln!("lab 0 error: {}", message);
    process::exit(65);
}


fn main(){
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        fail("Usage: lab0 <source-file>");
    }

    let filepath = &args[1];

    let contents = fs::read_to_string(filepath).
    unwrap_or_else(|error| fail("Failed to read file: {}"));

    print!("{contents}");
}