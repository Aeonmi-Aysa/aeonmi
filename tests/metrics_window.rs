use std::{iter::Peekable, process::Command, str::Chars};

fn run(args: &[&str]) -> (i32,String) {
    let out = Command::new(env!("CARGO_BIN_EXE_aeonmi")).args(args).output().expect("run");
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).to_string())
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

fn extract_json(output: &str) -> Option<String> {
    let cleaned = sanitize_output(output);
    let start = cleaned.find('{')?;
    let end = cleaned.rfind('}')?;
    Some(cleaned[start..=end].to_string())
}

#[test]
fn metrics_window_and_prune_behavior() {
    // Force small window
    std::env::set_var("AEONMI_EMA_ALPHA","50");
    std::env::set_var("AEONMI_METRICS_WINDOW","4");
    // Initial dump to create file
    let (_c,_o)= run(&["metrics-dump"]);
    // Simulate second dump (no activity) just to ensure fields exist
    let (_c2, json) = run(&["metrics-dump"]);
    let payload = extract_json(&json).expect("metrics dump json");
    let v: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert!(v.get("windowCapacity").is_some(), "missing windowCapacity");
    assert!(v.get("emaAlphaPct").is_some(), "missing emaAlphaPct");
    assert!(v.get("functionMetricsPruned").and_then(|x| x.as_u64()).is_some());
    // Savings structure baseline
    let savings = v.get("savings").expect("savings");
    assert!(savings.get("recent_window_partial_ns").is_some());
    assert!(savings.get("recent_window_estimated_full_ns").is_some());
    assert!(savings.get("recent_window_savings_pct").is_some());
    assert!(savings.get("recent_samples").is_some());
}
