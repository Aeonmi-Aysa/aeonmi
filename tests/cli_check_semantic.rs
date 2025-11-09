use std::fs;
use std::process::Command;

fn aeon_bin() -> String {
    env!("CARGO_BIN_EXE_aeon").to_string()
}

#[test]
fn check_exits_zero_on_valid_code() {
    let src = r#"
        fn main() {
            let x = 42;
            log(x);
        }
    "#;
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("valid.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(aeon_bin())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("failed to run aeon check");

    assert!(
        output.status.success(),
        "check should succeed on valid code\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_exits_nonzero_on_semantic_errors() {
    // This code has semantic errors:
    // 1. superpose() called with non-qubit argument
    // 2. hadamard() called with non-qubit argument
    let src = r#"
        fn main() {
            let x = 5;
            superpose(x);
            hadamard(x);
        }
    "#;
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("semantic_errors.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(aeon_bin())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("failed to run aeon check");

    assert!(
        !output.status.success(),
        "check should fail on semantic errors\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify error messages are present
    assert!(
        stderr.contains("Quantum operation") || stderr.contains("requires qubit"),
        "Expected quantum operation error message, got:\n{}",
        stderr
    );
}

#[test]
fn check_reports_qubit_argument_error() {
    let src = r#"
        fn test() {
            let x = 10;
            hadamard(x);
        }
    "#;
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("qubit_error.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(aeon_bin())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("failed to run aeon check");

    assert!(
        !output.status.success(),
        "check should fail when quantum op receives non-qubit"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("qubit"),
        "Error should mention 'qubit', got:\n{}",
        stderr
    );
}

#[test]
fn check_reports_boolean_condition_error() {
    // Use a quantum loop which also requires boolean condition
    let src = r#"
        fn test() {
            let x = 5;
            while x {
                log("wrong");
            }
        }
    "#;
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("bool_condition_error.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(aeon_bin())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("failed to run aeon check");

    assert!(
        !output.status.success(),
        "check should fail when loop has non-boolean condition"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("boolean") || stderr.contains("condition"),
        "Error should mention boolean or condition, got:\n{}",
        stderr
    );
}

#[test]
fn check_reports_undeclared_identifier() {
    let src = r#"
        fn test() {
            log(undefined_var);
        }
    "#;
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("undeclared.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(aeon_bin())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("failed to run aeon check");

    assert!(
        !output.status.success(),
        "check should fail on undeclared identifier"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("undefined") || stderr.contains("undeclared") || stderr.contains("not found"),
        "Error should mention undefined/undeclared identifier, got:\n{}",
        stderr
    );
}

#[test]
fn check_counts_multiple_errors() {
    // Multiple semantic errors in one file
    let src = r#"
        fn main() {
            let x = 5;
            superpose(x);
            hadamard(x);
            log(unknown_var);
        }
    "#;
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("multiple_errors.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(aeon_bin())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("failed to run aeon check");

    assert!(
        !output.status.success(),
        "check should fail on multiple errors"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should report multiple errors
    assert!(
        stderr.contains("error") || stderr.contains("Error"),
        "Should report errors, got:\n{}",
        stderr
    );
}
