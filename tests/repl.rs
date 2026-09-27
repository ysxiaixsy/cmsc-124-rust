use std::io::Write;
use std::process::{Command, Stdio};

fn run_repl(input: &str) -> (String, String, i32) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cmsc-124-rust"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start REPL");
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().expect("read REPL output");
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
        output.status.code().unwrap(),
    )
}

#[test]
fn repl_scans_each_line_without_a_blank_line() {
    let (stdout, stderr, code) = run_repl("42\n");
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
    assert!(stdout.starts_with("> Token {"), "{stdout}");
    assert!(stdout.contains("Number(\n            42.0,"), "{stdout}");
}

#[test]
fn repl_keeps_going_after_an_error() {
    let (stdout, stderr, code) = run_repl("@\n42\n");
    assert_eq!(code, 0);
    assert!(stderr.contains("Unexpected character '@' on line 1"), "{stderr}");
    assert!(stdout.contains("42.0"), "{stdout}");
}

#[test]
fn repl_preserves_spaces_inside_a_string() {
    let (stdout, stderr, code) = run_repl("\"hi  \"\n");
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
    assert!(stdout.contains("\"hi  \""), "{stdout}");
}
