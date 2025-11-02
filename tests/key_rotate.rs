use std::{fs, iter::Peekable, process::Command, str::Chars};

// Helper to run cli (aeonmi) with args returning stdout
fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_aeonmi")).args(args).output().expect("run aeonmi");
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).to_string(), String::from_utf8_lossy(&out.stderr).to_string())
}

fn sanitize_output(output: &str) -> String {
    fn strip_sequence(chars: &mut Peekable<Chars<'_>>, terminator: impl Fn(char) -> bool) {
        while let Some(c) = chars.next() {
            if terminator(c) {
                break;
            }
        }
    }

    let mut result = String::with_capacity(output.len());
    let mut chars = output.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            match chars.peek().copied() {
                Some('[') => {
                    chars.next();
                    strip_sequence(&mut chars, |c| c >= '@' && c <= '~');
                }
                Some(']') => {
                    chars.next();
                    strip_sequence(&mut chars, |c| c == '\u{7}');
                }
                _ => {}
            }
        } else {
            result.push(ch);
        }
    }
    result
}

fn last_value_line(output: &str) -> String {
    let cleaned = sanitize_output(output);
    cleaned.lines().rev().find(|line| !line.trim().is_empty()).unwrap_or("").trim().to_string()
}

fn extract_json(output: &str) -> Option<String> {
    let cleaned = sanitize_output(output);
    let start = cleaned.find('{')?;
    let end = cleaned.rfind('}')?;
    Some(cleaned[start..=end].to_string())
}

#[test]
fn key_rotation_preserves_plaintext() {
    // Use temp config dir to isolate keys file
    let td = tempfile::tempdir().unwrap();
    let config_dir = td.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    std::env::set_var("AEONMI_CONFIG_DIR", config_dir.to_str().unwrap());
    // Set a key
    let (_c,_o,_e)= run(&["key-set","testprov","ABC123"]);
    // Get it
    let (_c2,o2,_e2)= run(&["key-get","testprov"]);
    assert_eq!(last_value_line(&o2), "ABC123");
    // Rotate in JSON mode
    let (_c3,o3,_e3)= run(&["key-rotate","--json"]);
    let json = extract_json(&o3).expect("json payload");
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("rotated").and_then(|x| x.as_u64()).unwrap_or(0) >= 1);
    // Get again
    let (_c4,o4,_e4)= run(&["key-get","testprov"]);
    std::env::remove_var("AEONMI_CONFIG_DIR");
    assert_eq!(last_value_line(&o4), "ABC123");
}
