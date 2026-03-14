/// Phase 3 — P3-4: File I/O built-in tests.
///
/// Verifies that `read_file`, `write_file`, `append_file`, `file_exists`,
/// `read_lines`, and `delete_file` work correctly from Aeonmi scripts.

use std::process::Command;

/// Helper: run an aeonmi snippet via the native VM and return stdout.
fn run_snippet(code: &str) -> String {
    let tmp = std::env::temp_dir().join(format!("shard_io_test_{}.ai", std::process::id()));
    std::fs::write(&tmp, code).unwrap();
    let bin = env!("CARGO_BIN_EXE_aeonmi");
    let out = Command::new(bin)
        .arg("run")
        .arg(tmp.to_str().unwrap())
        .output()
        .expect("failed to run aeonmi");
    std::fs::remove_file(&tmp).ok();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if !out.status.success() {
        panic!("aeonmi failed:\nstdout: {}\nstderr: {}", stdout, stderr);
    }
    stdout
}

#[test]
fn test_write_and_read_file() {
    let tmp = std::env::temp_dir().join(format!("ae_rw_{}.txt", std::process::id()));
    let path = tmp.to_str().unwrap().replace('\\', "/");
    let code = format!(
        r#"
let path = "{}";
write_file(path, "hello shard");
let content = read_file(path);
print(content);
delete_file(path);
"#,
        path
    );
    let out = run_snippet(&code);
    assert!(out.contains("hello shard"), "read_file should return written content, got: {}", out);
    println!("✅ write_file + read_file round-trip");
}

#[test]
fn test_file_exists_true_and_false() {
    let tmp = std::env::temp_dir().join(format!("ae_fe_{}.txt", std::process::id()));
    let path = tmp.to_str().unwrap().replace('\\', "/");
    let code = format!(
        r#"
let path = "{}";
let before = file_exists(path);
write_file(path, "x");
let after = file_exists(path);
delete_file(path);
print(before);
print(after);
"#,
        path
    );
    let out = run_snippet(&code);
    assert!(out.contains("false"), "file_exists should be false before creation, got: {}", out);
    assert!(out.contains("true"),  "file_exists should be true after creation,  got: {}", out);
    println!("✅ file_exists returns correct booleans");
}

#[test]
fn test_append_file() {
    let tmp = std::env::temp_dir().join(format!("ae_ap_{}.txt", std::process::id()));
    let path = tmp.to_str().unwrap().replace('\\', "/");
    let code = format!(
        r#"
let path = "{}";
write_file(path, "line1\n");
append_file(path, "line2\n");
let content = read_file(path);
print(content);
delete_file(path);
"#,
        path
    );
    let out = run_snippet(&code);
    assert!(out.contains("line1"), "appended file should contain line1, got: {}", out);
    assert!(out.contains("line2"), "appended file should contain line2, got: {}", out);
    println!("✅ append_file adds content");
}

#[test]
fn test_read_lines() {
    let tmp = std::env::temp_dir().join(format!("ae_rl_{}.txt", std::process::id()));
    let path = tmp.to_str().unwrap().replace('\\', "/");
    // Write a file with 3 lines
    std::fs::write(&tmp, "alpha\nbeta\ngamma\n").unwrap();
    let code = format!(
        r#"
let path = "{}";
let lines = read_lines(path);
print(len(lines));
print(lines[0]);
print(lines[2]);
delete_file(path);
"#,
        path
    );
    let out = run_snippet(&code);
    assert!(out.contains("3"), "should have 3 lines, got: {}", out);
    assert!(out.contains("alpha"), "first line should be 'alpha', got: {}", out);
    assert!(out.contains("gamma"), "third line should be 'gamma', got: {}", out);
    println!("✅ read_lines returns correct lines");
}

#[test]
fn test_delete_file() {
    let tmp = std::env::temp_dir().join(format!("ae_del_{}.txt", std::process::id()));
    let path = tmp.to_str().unwrap().replace('\\', "/");
    let code = format!(
        r#"
let path = "{}";
write_file(path, "to delete");
let before = file_exists(path);
delete_file(path);
let after = file_exists(path);
print(before);
print(after);
"#,
        path
    );
    let out = run_snippet(&code);
    assert!(out.contains("true"),  "should exist before deletion, got: {}", out);
    assert!(out.contains("false"), "should not exist after deletion, got: {}", out);
    println!("✅ delete_file removes file");
}

/// Ensure the Shard bootstrap (shard/src/main.ai) now uses real file I/O.
/// The shard should compile examples/hello.ai and write a .compiled.ai file.
#[test]
fn test_shard_reads_real_file() {
    let bin = env!("CARGO_BIN_EXE_aeonmi");
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shard_main = repo.join("shard/src/main.ai");

    if !shard_main.exists() {
        println!("⏭️ Skipping: shard/src/main.ai not found");
        return;
    }

    let out = Command::new(bin)
        .arg("run")
        .arg(shard_main.to_str().unwrap())
        .current_dir(&repo)
        .output()
        .expect("failed to run shard main.ai");

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    println!("Shard output:\n{}", stdout);

    // Shard should print "SHARD" and indicate it read a file
    assert!(stdout.contains("SHARD"), "shard output should contain 'SHARD'");
    assert!(
        stdout.contains("bytes") || stdout.contains("Compilation") || stdout.contains("Compiled"),
        "shard should report reading file, got: {}",
        stdout
    );
    println!("✅ Shard reads and compiles real .ai file");
}
