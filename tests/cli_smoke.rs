use std::fs;
use std::process::Command;

fn bin() -> String {
    // Cargo sets this environment variable for binary targets in integration tests
    env!("CARGO_BIN_EXE_aeonmi_project").to_string()
}

/// Test that `--format` (visible alias for the emit kind parameter) works with the `emit` subcommand.
#[test]
fn emit_subcommand_format_alias_produces_js() {
    let src = "let y = 10; log(y);";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("alias.ai");
    let out = dir.path().join("alias_out.js");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .args([
            "emit",
            input.to_str().unwrap(),
            "--format",
            "js",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run aeonmi_project");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let js = fs::read_to_string(&out).expect("output file should exist when --format js used");
    assert!(
        js.contains("console.log(y)") || js.contains("log(y)"),
        "emitted JS missing expected log call; got:\n{js}"
    );
}

/// Test that value alias `js` (lower-case) resolves for `--emit`.
#[test]
fn emit_value_alias_js_works_via_legacy_flag() {
    let src = "let z = 42; log(z);";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("valias.ai");
    let out = dir.path().join("valias_out.js");
    fs::write(&input, src).unwrap();

    // Use legacy top-level `--emit js` flag (the alias `js` for EmitKind::Js)
    let output = Command::new(bin())
        .args([
            "--emit",
            "js",
            "--out",
            out.to_str().unwrap(),
            input.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run aeonmi_project");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let js = fs::read_to_string(&out).expect("output file should exist when --emit js used");
    assert!(
        js.contains("42") || js.contains("console.log") || js.contains("log(z)"),
        "emitted JS missing expected content; got:\n{js}"
    );
}

/// Test that value alias `ai` for EmitKind::Ai produces an .ai output file.
#[test]
fn emit_value_alias_ai_works_via_legacy_flag() {
    let src = "let a = 1;";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("ai_alias.ai");
    let out = dir.path().join("ai_alias_out.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .args([
            "--emit",
            "ai",
            "--out",
            out.to_str().unwrap(),
            input.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run aeonmi_project");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(out.exists(), ".ai output file should exist when --emit ai used");
}

#[test]
fn cli_compiles_basic_file() {
    let src = r#"
        let x = 2 + 3;
        log(x);
    "#;
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("ok.ai");
    let out = dir.path().join("out.js");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .arg("--tokens")
        .arg("--ast")
        .arg("--emit")
        .arg("js")
        .arg("--out")
        .arg(out.to_str().unwrap())
        .arg(input.to_str().unwrap())
        .output()
        .expect("failed to run aeonmi_project");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let js = fs::read_to_string(&out).expect("output file should exist");
    assert!(
        js.contains("let x = (2 + 3);") || js.contains("let x = 2 + 3;"),
        "output JS missing expected code"
    );
    assert!(
        js.contains("console.log(x);"),
        "output JS missing expected console.log call"
    );
}

#[test]
fn cli_skips_semantic_when_flagged() {
    let src = "let x = 1; log(x);";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("ok.ai");
    let out = dir.path().join("out.js");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .arg("--no-sema")
        .arg("--out")
        .arg(out.to_str().unwrap())
        .arg(input.to_str().unwrap())
        .output()
        .expect("failed to run aeonmi_project");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Accept either stream; case-insensitive and allow minor phrasing differences
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .to_lowercase();

    assert!(
        combined.contains("semantic analyzer: skipped")
            || combined.contains("skipped by flag")
            || (combined.contains("semantic analyzer") && combined.contains("skipp"))
            || combined.contains("semantic analysis skipped"),
        "did not find expected skip message in output:\n{}",
        combined
    );
}

#[test]
fn cli_rejects_unsupported_emit() {
    let src = "let x = 1; log(x);";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("ok.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .arg("--emit")
        .arg("wasm")
        .arg(input.to_str().unwrap())
        .output()
        .expect("failed to run aeonmi_project");

    assert!(
        !output.status.success(),
        "unexpected success running with unsupported emit kind"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unsupported --emit kind"),
        "stderr did not contain expected error message"
    );
}
