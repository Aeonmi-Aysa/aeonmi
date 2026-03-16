use std::fs;
use std::process::Command;

fn bin() -> String {
    env!("CARGO_BIN_EXE_aeonmi_project").to_string()
}

#[test]
fn pretty_lexer_error_shows_span() {
    let bad = r#"let x = "unterminated;"#; // missing closing quote
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("bad_lexer.ai");
    fs::write(&input, bad).unwrap();

    let output = Command::new(bin())
        .arg("--pretty-errors")
        .arg(input.to_str().unwrap())
        .output()
        .expect("run");

    assert!(!output.status.success());
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("error:"), "no 'error:' in stderr\n{err}");
    assert!(
        err.contains("Unterminated string"),
        "no message in stderr\n{err}"
    );
    // accept any line/col, but file + colon must be present
    assert!(
        err.contains("bad_lexer.ai:"),
        "no file:line:col in stderr\n{err}"
    );
}

#[test]
fn pretty_parser_error_shows_span() {
    // missing '(' after function name — a clear parser error
    let bad = "function foo {";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("bad_parser.ai");
    fs::write(&input, bad).unwrap();

    let output = Command::new(bin())
        .arg("--pretty-errors")
        .arg(input.to_str().unwrap())
        .output()
        .expect("run");

    assert!(!output.status.success());
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("error:"), "no 'error:' in stderr\n{err}");
    assert!(
        err.to_lowercase().contains("expected '('"),
        "parser message missing\n{err}"
    );
    // span must include the filename
    assert!(
        err.contains("bad_parser.ai:"),
        "no file prefix in span\n{err}"
    );
}
