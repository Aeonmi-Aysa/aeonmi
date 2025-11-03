// Integration test: ensures metrics JSON includes new schema v5 fields.
// Runs `aeonmi metrics-dump` after ensuring metrics file exists.

use std::{iter::Peekable, process::Command, str::Chars};

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

fn extract_json(output: &str) -> Option<String> {
    let cleaned = sanitize_output(output);
    let start = cleaned.find('{')?;
    let end = cleaned.rfind('}')?;
    Some(cleaned[start..=end].to_string())
}

#[test]
fn metrics_json_includes_v5_fields() {
    // Run metrics-dump
    let output = Command::new(env!("CARGO_BIN_EXE_aeonmi"))
        .arg("metrics-dump")
        .output()
        .expect("failed to run aeonmi metrics-dump");
    assert!(output.status.success(), "metrics-dump did not succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload = extract_json(&stdout).expect("metrics dump json");
    let val: serde_json::Value = serde_json::from_str(&payload).expect("valid JSON");
    let version = val.get("version").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(
        version >= 5,
        "expected metrics version >=5, got {}",
        version
    );
    let savings = val
        .get("savings")
        .and_then(|v| v.as_object())
        .expect("savings object");
    assert!(
        savings.contains_key("cumulative_savings_pct"),
        "missing cumulative_savings_pct"
    );
    assert!(
        savings.contains_key("cumulative_partial_pct"),
        "missing cumulative_partial_pct"
    );
    // functionMetrics should be an object
    val.get("functionMetrics")
        .and_then(|v| v.as_object())
        .expect("functionMetrics object");
}
