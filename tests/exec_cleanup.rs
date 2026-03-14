use std::{fs, process::Command};

fn bin() -> String { env!("CARGO_BIN_EXE_aeonmi_project").to_string() }

#[test]
fn exec_ai_native_no_temp_js() {
    // exec for .ai always uses the native VM — no __exec_tmp.js is produced.
    let ai = "temp_cleanup.ai";
    fs::write(ai, "let a = 1;\nlog(a);\n").unwrap();

    // Clean up any stale artifact from prior runs
    let _ = fs::remove_file("__exec_tmp.js");

    let status = Command::new(bin()).args(["exec", ai]).status().unwrap();
    assert!(status.success());
    assert!(!std::path::Path::new("__exec_tmp.js").exists(), "exec .ai must not produce __exec_tmp.js");
    let _ = fs::remove_file(ai);
}

#[test]
fn exec_ai_no_run_flag_succeeds() {
    // --no-run is a compile-only signal; native path parses but doesn't execute.
    let ai = "temp_keep.ai";
    fs::write(ai, "let a = 2;\nlog(a);\n").unwrap();
    let status = Command::new(bin()).args(["exec", ai, "--no-run"]).status().unwrap();
    assert!(status.success());
    // No JS artifact expected in the native path
    assert!(!std::path::Path::new("__exec_tmp.js").exists(), "exec .ai should not produce __exec_tmp.js even with --no-run");
    let _ = fs::remove_file(ai);
}
