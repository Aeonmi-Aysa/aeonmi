/// Phase 5 — IDEA 1: Smart-Contract Verifier integration tests.
///
/// Verifies the `aeonmi verify` command line interface and verifier engine.

use std::process::Command;

/// Helper: run an aeonmi verify command and return stdout + exit status.
fn run_verify(file_path: &str, extra_args: &[&str]) -> (String, bool) {
    let bin = env!("CARGO_BIN_EXE_aeonmi");
    let mut cmd = Command::new(bin);
    cmd.arg("verify").arg(file_path);
    for arg in extra_args {
        cmd.arg(arg);
    }
    let out = cmd.output().expect("failed to run aeonmi verify");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if !stdout.is_empty() {
        eprintln!("stdout: {}", stdout);
    }
    if !stderr.is_empty() {
        eprintln!("stderr: {}", stderr);
    }
    (stdout, out.status.success())
}

#[test]
fn test_verify_pure_contract() {
    let tmp = std::env::temp_dir().join(format!("ae_verify_pure_{}.ai", std::process::id()));
    std::fs::write(&tmp, r#"
function add(a, b) {
    return a + b;
}

function multiply(x, y) {
    return x * y;
}
"#).unwrap();

    let (output, success) = run_verify(tmp.to_str().unwrap(), &[]);
    assert!(success, "verify should succeed");
    assert!(output.contains("Pure (constant)") || output.contains("constant (pure)"),
        "Should identify pure functions, got: {}", output);
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn test_verify_json_output() {
    let tmp = std::env::temp_dir().join(format!("ae_verify_json_{}.ai", std::process::id()));
    std::fs::write(&tmp, r#"
function compute(x) {
    return x * 2;
}
"#).unwrap();

    let (output, success) = run_verify(tmp.to_str().unwrap(), &["--json"]);
    assert!(success, "verify --json should succeed");
    assert!(output.contains("\"file\""), "JSON should contain file field, got: {}", output);
    assert!(output.contains("\"functions\""), "JSON should contain functions field");
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn test_verify_stateful_contract() {
    let tmp = std::env::temp_dir().join(format!("ae_verify_stateful_{}.ai", std::process::id()));
    std::fs::write(&tmp, r#"
function save(path, data) {
    write_file(path, data);
}
"#).unwrap();

    let (output, success) = run_verify(tmp.to_str().unwrap(), &["--json"]);
    assert!(success, "verify should succeed");
    assert!(output.contains("balanced"), "Should detect stateful function");
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn test_verify_report_to_file() {
    let tmp = std::env::temp_dir().join(format!("ae_verify_out_{}.ai", std::process::id()));
    let out = std::env::temp_dir().join(format!("ae_verify_report_{}.json", std::process::id()));
    std::fs::write(&tmp, r#"
function helper(x) {
    return x + 1;
}
"#).unwrap();

    let (_, success) = run_verify(tmp.to_str().unwrap(), &["--out", out.to_str().unwrap()]);
    assert!(success, "verify --out should succeed");
    assert!(out.exists(), "Report file should be created");
    let report = std::fs::read_to_string(&out).unwrap();
    assert!(report.contains("helper"), "Report should contain function name");
    std::fs::remove_file(&tmp).ok();
    std::fs::remove_file(&out).ok();
}
