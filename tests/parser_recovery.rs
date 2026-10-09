use std::process::Command;

#[test]
fn consecutive_syntax_errors_keep_the_next_line_available() {
    let source = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/lab2/errors/recovery_consecutive_lines.trap"
    );
    let output = Command::new(env!("CARGO_BIN_EXE_cmsc-124-rust"))
        .args(["--parse", source])
        .output()
        .expect("run parser");

    assert_eq!(output.status.code(), Some(65));
    assert!(
        output.stdout.is_empty(),
        "rejected files must not print trees"
    );
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 diagnostics");
    assert_eq!(
        stderr.lines().collect::<Vec<_>>(),
        [
            "[line 1] Error at end: Expect expression.",
            "[line 2] Error at end: Expect expression.",
            "[line 3] Error at end: Expect expression.",
        ]
    );
}
