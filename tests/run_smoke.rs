use std::process::Command;

#[test]
fn run_subcommand_executes_with_native_vm() {
    // `run` uses the native VM interpreter — no JS file is produced.
    let output = Command::new(env!("CARGO_BIN_EXE_aeonmi_project"))
        .args(["run", "examples/hello.ai"])
        .output()
        .expect("failed to spawn");
    assert!(output.status.success(), "run subcommand should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Native execution always prints the "native: executing" marker
    assert!(
        stdout.contains("native: executing"),
        "expected native execution marker in stdout: {stdout}"
    );
    // Ensure no JS artifact is written
    assert!(
        !std::path::Path::new("aeonmi.run.js").exists(),
        "run must not emit aeonmi.run.js"
    );
}
