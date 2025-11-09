use std::process::Command;

fn bin() -> String {
    env!("CARGO_BIN_EXE_aeonmi_project").to_string()
}

#[test]
fn shell_run_native_skips_js_emit() {
    // Launch shell with a temp dir
    let dir = tempfile::tempdir().unwrap();
    let ai_path = dir.path().join("demo.ai");
    std::fs::write(&ai_path, "let a = 5; let b = a * 2; log(b);").unwrap();

    let file_arg = ai_path.to_string_lossy().into_owned();

    let output = Command::new(bin())
        .current_dir(dir.path())
        .arg("run")
        .arg("--native")
        .arg(&file_arg)
        .output()
        .expect("native shell run");
    assert!(
        output.status.success(),
        "shell exit status not success: stdout=\n{}\nstderr=\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Program executed successfully"),
        "expected native success message in stdout, stdout=\n{}\nstderr=\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !dir.path().join("aeonmi.run.js").exists(),
        "unexpected JS file emitted in shell native run mode"
    );
}
