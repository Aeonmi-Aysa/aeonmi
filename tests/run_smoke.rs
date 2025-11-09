use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn run_subcommand_compiles_even_without_node() {
    let out = "aeonmi.run.js";
    let output = Command::new(env!("CARGO_BIN_EXE_aeonmi_project"))
        .env("AEONMI_USE_JS", "1")
        .args(["run", "--output", out, "examples/hello.ai"])
        .output()
        .expect("failed to spawn");

    assert!(
        output.status.success(),
        "run subcommand should succeed: stdout=\n{}\nstderr=\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let js_path = Path::new(out);
    if js_path.exists() {
        fs::remove_file(js_path).ok();
    } else {
        assert!(
            stdout.contains("Program executed successfully")
                || stderr.contains("Node.js not found"),
            "expected JS artifact or explicit native fallback notice, stdout=\n{}\nstderr=\n{}",
            stdout,
            stderr
        );
    }
}
