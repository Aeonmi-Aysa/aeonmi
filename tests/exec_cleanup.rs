use std::{
    fs,
    path::Path,
    process::Command,
    sync::{Mutex, OnceLock},
};

fn exec_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn bin() -> String {
    env!("CARGO_BIN_EXE_aeonmi_project").to_string()
}

#[test]
fn exec_ai_removes_temp_by_default() {
    let _guard = exec_guard();
    let ai = "temp_cleanup.ai";
    fs::write(ai, "let a = 1;\nlog(a);\n").unwrap();

    // Clean up any existing temp file first
    let _ = fs::remove_file("__exec_tmp.js");

    let status = Command::new(bin())
        .args(["exec", ai, "--no-run"])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(
        !Path::new("__exec_tmp.js").exists(),
        "temp js should be removed"
    );
    let _ = fs::remove_file(ai);
}

#[test]
fn exec_ai_keeps_temp_with_flag() {
    let _guard = exec_guard();
    let ai = "temp_keep.ai";
    let _ = fs::remove_file("__exec_tmp.js");
    fs::write(ai, "let a = 2;\nlog(a);\n").unwrap();
    let status = Command::new(bin())
        .args(["exec", ai, "--keep-temp", "--no-run"])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(
        Path::new("__exec_tmp.js").exists(),
        "temp js should remain when --keep-temp"
    );
    let _ = fs::remove_file("__exec_tmp.js");
    let _ = fs::remove_file(ai);
}
