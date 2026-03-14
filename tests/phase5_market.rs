/// Phase 5 — IDEA 3: Genesis Glyph NFT Marketplace integration tests.
///
/// Tests the `aeonmi market` CLI subcommands.

use std::process::Command;

/// Helper: run an aeonmi market command and return stdout + success status.
fn run_market(args: &[&str]) -> (String, bool) {
    let bin = env!("CARGO_BIN_EXE_aeonmi");
    let mut cmd = Command::new(bin);
    cmd.arg("market");
    for arg in args {
        cmd.arg(arg);
    }
    let out = cmd.output().expect("failed to run aeonmi market");
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
fn test_market_glyphs() {
    let (output, success) = run_market(&["glyphs"]);
    assert!(success, "market glyphs should succeed");
    assert!(output.contains("G-1"), "Should list G-1 glyph");
    assert!(output.contains("G-12"), "Should list G-12 glyph");
    assert!(output.contains("⊗"), "Should show tensor product symbol");
    assert!(output.contains("⊕"), "Should show XOR symbol");
}

#[test]
fn test_market_list_empty_dir() {
    let tmp = std::env::temp_dir().join(format!("ae_market_empty_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let (output, success) = run_market(&["list", tmp.to_str().unwrap()]);
    assert!(success, "market list should succeed on empty dir");
    assert!(output.contains("No .qube circuits") || output.contains("Total circuits: 0"),
        "Should indicate no circuits found, got: {}", output);
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_market_list_with_qube_file() {
    let tmp = std::env::temp_dir().join(format!("ae_market_qube_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("bell.qube"), "state q0 = |0⟩\nstate q1 = |0⟩\napply H → q0\ncollapse q0 → r0\n").unwrap();

    let (output, success) = run_market(&["list", tmp.to_str().unwrap()]);
    assert!(success, "market list should succeed");
    assert!(output.contains("bell") || output.contains("Total circuits: 1"),
        "Should find bell.qube, got: {}", output);
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_market_info_qube_file() {
    let tmp = std::env::temp_dir().join(format!("ae_market_info_{}.qube", std::process::id()));
    std::fs::write(&tmp, "state q = |0⟩\napply H → q\ncollapse q → result\n").unwrap();

    let (output, success) = run_market(&["info", tmp.to_str().unwrap()]);
    assert!(success, "market info should succeed");
    assert!(output.contains("Qubits") || output.contains("qubit"),
        "Should show qubit info, got: {}", output);
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn test_market_mint_qube_file() {
    let tmp = std::env::temp_dir().join(format!("ae_market_mint_{}.qube", std::process::id()));
    std::fs::write(&tmp, "state q = |0⟩\napply H → q\ncollapse q → result\n").unwrap();

    let (output, success) = run_market(&["mint", tmp.to_str().unwrap()]);
    assert!(success, "market mint should succeed");
    assert!(output.contains("source_hash") || output.contains("quantum_signature"),
        "Should output NFT metadata JSON, got: {}", output);
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn test_market_list_json() {
    let tmp = std::env::temp_dir().join(format!("ae_market_json_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("test.qube"), "state q = |0⟩\napply X → q\ncollapse q → r\n").unwrap();

    let (output, success) = run_market(&["list", tmp.to_str().unwrap(), "--json"]);
    assert!(success, "market list --json should succeed");
    assert!(output.contains("[") && output.contains("]"), "JSON output should contain an array, got: {}", output);
    std::fs::remove_dir_all(&tmp).ok();
}
