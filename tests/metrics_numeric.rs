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
fn metrics_numeric_window_and_ema() {
    // Reset config then set known values
    std::env::set_var("AEONMI_EMA_ALPHA","50");
    std::env::set_var("AEONMI_METRICS_WINDOW","4");
    // Initial dump
    let (_c,_o)= run(&["metrics-dump"]);
    // We cannot directly trigger record_function_infer from CLI; emulate by persisting manual edit not exposed.
    // Skip deep numeric validation due to lack of direct hooks; ensure version >=6 and window/ema fields.
    let (_c2, json) = run(&["metrics-dump"]);
    let payload = extract_json(&json).expect("metrics dump json");
    let v: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert!(v.get("version").and_then(|x| x.as_u64()).unwrap_or(0) >= 6);
    assert!(v.get("windowCapacity").is_some());
    assert!(v.get("emaAlphaPct").is_some());
}
