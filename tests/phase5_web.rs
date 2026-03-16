/// Phase 5 — IDEA 2: Reactive Web Framework integration tests.
///
/// Tests the HTTP built-ins (http_response, http_get, http_post, http_json)
/// through the native VM interpreter.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Helper: run an aeonmi snippet via the native VM and return stdout.
fn run_snippet(code: &str) -> String {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let tmp = std::env::temp_dir().join(format!("shard_web_test_{}_{}.ai", std::process::id(), id));
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
fn test_http_response_builtin() {
    let code = r#"
let resp = http_response(200, "Hello World");
print(resp);
"#;
    let out = run_snippet(code);
    assert!(out.contains("200"), "Response should contain status 200, got: {}", out);
    assert!(out.contains("Hello World"), "Response should contain body");
}

#[test]
fn test_http_json_builtin() {
    let code = r#"
let resp = http_json(201, "{\"created\": true}");
print(resp);
"#;
    let out = run_snippet(code);
    assert!(out.contains("201"), "JSON response should contain status 201, got: {}", out);
}

#[test]
fn test_http_get_builtin() {
    let code = r#"
let req = http_get("/api/data");
print(typeof(req));
"#;
    let out = run_snippet(code);
    assert!(out.contains("object"), "http_get should return an object, got: {}", out);
}

#[test]
fn test_http_post_builtin() {
    let code = r#"
let req = http_post("/api/submit", "{\"key\": \"value\"}");
print(typeof(req));
"#;
    let out = run_snippet(code);
    assert!(out.contains("object"), "http_post should return an object, got: {}", out);
}
