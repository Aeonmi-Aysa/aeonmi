use std::path::Path;
use std::process::Command;

#[test]
fn run_subcommand_executes_natively() {
    // `run` always uses the native interpreter — no JS output file is produced.
    let output = Command::new(env!("CARGO_BIN_EXE_aeonmi_project"))
        .args(["run", "examples/hello.ai"])
        .output()
        .expect("failed to spawn");
    assert!(output.status.success(), "run subcommand should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("native: executing"), "run should use native interpreter");
    // Ensure no JS artifact is written
    assert!(!Path::new("aeonmi.run.js").exists(), "run must not emit aeonmi.run.js");
}
