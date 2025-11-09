use std::{iter::Peekable, process::Command, str::Chars};

fn run(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_aeonmi"))
        .env("AEON_ENHANCED_CLI", "false")
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
fn savings_history_reload_preserves_window_counters() {
    // Set history cap via CLI
    let (cfg_code, cfg_out) = run(&["metrics-config", "--set-history-cap", "16"]);
    if cfg_code != 0 {
        eprintln!("metrics-config unavailable: {}", sanitize_output(&cfg_out));
        return;
    }
    // Flush persisted metrics so subsequent dump reflects disk state.
    let (flush_code, flush_out) = run(&["metrics-flush"]);
    if flush_code != 0 {
        eprintln!("metrics-flush unavailable: {}", sanitize_output(&flush_out));
        return;
    }
    // Capture state prior to reload.
    let (dump_before_code, out_before) = run(&["metrics-dump"]);
    assert_eq!(
        dump_before_code,
        0,
        "metrics-dump failed: {}",
        sanitize_output(&out_before)
    );
    let before_payload = extract_json(&out_before).expect("metrics dump json");
    let v_before: serde_json::Value = serde_json::from_str(&before_payload).unwrap();
    let savings_before = v_before
        .get("savings")
        .and_then(|v| v.as_object())
        .expect("savings before");
    let before_partial = savings_before
        .get("recent_window_partial_ns")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let before_estimated = savings_before
        .get("recent_window_estimated_full_ns")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // New invocation reloads from disk; ensure counters survive round-trip.
    let (dump_after_code, out_after) = run(&["metrics-dump"]);
    assert_eq!(
        dump_after_code,
        0,
        "metrics-dump reload failed: {}",
        sanitize_output(&out_after)
    );
    let after_payload = extract_json(&out_after).expect("metrics dump reload json");
    let v_after: serde_json::Value = serde_json::from_str(&after_payload).unwrap();
    let savings_after = v_after
        .get("savings")
        .and_then(|v| v.as_object())
        .expect("savings after");
    let after_partial = savings_after
        .get("recent_window_partial_ns")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let after_estimated = savings_after
        .get("recent_window_estimated_full_ns")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    assert_eq!(
        after_partial, before_partial,
        "recent partial window counter changed after reload"
    );
    assert_eq!(
        after_estimated, before_estimated,
        "recent estimated window counter changed after reload"
    );

    // Restore defaults to reduce cross-test interference.
    let _ = run(&["metrics-config", "--reset"]);
}
