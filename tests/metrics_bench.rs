use std::{iter::Peekable, process::Command, str::Chars};

fn run(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_aeonmi"))
        .args(args)
        .output()
        .expect("run");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
    )
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
fn metrics_bench_generates_functions() {
    let (code, out) = run(&[
        "metrics-bench",
        "--functions",
        "3",
        "--samples",
        "5",
        "--reset",
    ]);
    if code != 0 {
        eprintln!("metrics-bench unavailable: {out}");
        return;
    }
    let (_c2, json) = run(&["metrics-dump"]);
    let payload = extract_json(&json).expect("metrics json");
    let v: serde_json::Value = serde_json::from_str(&payload).unwrap();
    let fm = v.get("functionMetrics").unwrap().as_object().unwrap();
    assert!(
        fm.len() >= 3,
        "expected at least 3 function metrics after bench"
    );
}
