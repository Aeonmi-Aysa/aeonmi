use std::fs;
use std::process::Command;

fn bin() -> String {
    env!("CARGO_BIN_EXE_aeonmi_project").to_string()
}

#[test]
fn ai_review_clean_file_reports_no_issues() {
    let src = "let x = 2 + 3;\nlog(x);\n";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("clean.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .args(["ai", "review", "--file"])
        .arg(&input)
        .output()
        .expect("failed to run aeonmi_project");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no issues found"),
        "expected 'no issues found' in: {stdout}"
    );
}

#[test]
fn ai_review_detects_trailing_whitespace() {
    let src = "let x = 1;   \nlog(x);\n";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("trailing.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .args(["ai", "review", "--file"])
        .arg(&input)
        .output()
        .expect("failed to run aeonmi_project");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("trailing whitespace"),
        "expected trailing whitespace finding in: {stdout}"
    );
}

#[test]
fn ai_review_detects_missing_semicolon() {
    let src = "let y = 42\nlog(y);\n";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("nosemi.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .args(["ai", "review", "--file"])
        .arg(&input)
        .output()
        .expect("failed to run aeonmi_project");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("semicolon"),
        "expected semicolon finding in: {stdout}"
    );
}

#[test]
fn ai_review_suggest_flag_shows_suggestion() {
    let src = "let y = 42\nlog(y);\n";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("suggest.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .args(["ai", "review", "--file"])
        .arg(&input)
        .arg("--suggest")
        .output()
        .expect("failed to run aeonmi_project");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("suggestion:"),
        "expected suggestion text in: {stdout}"
    );
}

#[test]
fn ai_review_json_output() {
    let src = "let y = 42\nlog(y);\n";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("json_test.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .args(["ai", "review", "--file"])
        .arg(&input)
        .arg("--json")
        .output()
        .expect("failed to run aeonmi_project");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\"findings\""),
        "expected JSON findings key in: {stdout}"
    );
    assert!(
        stdout.contains("\"category\""),
        "expected JSON category key in: {stdout}"
    );
}

#[test]
fn ai_review_missing_file_arg_exits_nonzero() {
    let output = Command::new(bin())
        .args(["ai", "review"])
        .output()
        .expect("failed to run aeonmi_project");

    assert!(
        !output.status.success(),
        "expected non-zero exit when --file is omitted"
    );
}
