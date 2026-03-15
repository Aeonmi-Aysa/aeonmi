/// Phase 4 — P1-34 + Genesis Glyphs: feature tests
///
/// Verifies:
///   - `for x in collection` iterates correctly (P1-34)
///   - Genesis array literal ⧉ elem ‥ elem ⧉ parses and evaluates to an array
///   - GenesisBinding / spread tokenize correctly

use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::token::TokenKind;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

// ── CLI helper ────────────────────────────────────────────────────────────────

/// Write `code` to a unique temp file and run it through `aeonmi run`; return stdout.
fn run_snippet(code: &str) -> String {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let tmp = std::env::temp_dir().join(format!(
        "ae_p4_{}_{}.ai",
        std::process::id(),
        id
    ));
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

// ── P1-34: for-in proper iteration ───────────────────────────────────────────

#[test]
fn for_in_iterates_array() {
    let src = r#"
let items = [10, 20, 30];
let sum = 0;
for x in items {
    sum = sum + x;
}
print(sum);
"#;
    let out = run_snippet(src);
    assert!(
        out.contains("60"),
        "expected 60 in output (sum of 10+20+30), got: {:?}",
        out
    );
}

#[test]
fn for_in_iterates_string_chars() {
    let src = r#"
let s = "hi";
let count = 0;
for c in s {
    count = count + 1;
}
print(count);
"#;
    let out = run_snippet(src);
    assert!(
        out.contains('2'),
        "expected 2 (two chars in 'hi'), got: {:?}",
        out
    );
}

// ── Genesis Glyph tokenization ────────────────────────────────────────────────

#[test]
fn genesis_tokens_lex_correctly() {
    let src = "⧉ 1 ‥ 2 ⧉";
    let tokens = Lexer::from_str(src).tokenize().expect("lex error");
    let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
    assert!(
        kinds.contains(&&TokenKind::ArrayGenesisBracket),
        "expected ArrayGenesisBracket, got: {:?}", kinds
    );
    assert!(
        kinds.contains(&&TokenKind::GenesisSep),
        "expected GenesisSep, got: {:?}", kinds
    );
}

#[test]
fn genesis_binding_lex() {
    let src = "x ↦ y";
    let tokens = Lexer::from_str(src).tokenize().expect("lex error");
    let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
    assert!(
        kinds.contains(&&TokenKind::GenesisBinding),
        "expected GenesisBinding (↦), got: {:?}", kinds
    );
}

#[test]
fn genesis_spread_lex() {
    let src = "…items";
    let tokens = Lexer::from_str(src).tokenize().expect("lex error");
    let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
    assert!(
        kinds.contains(&&TokenKind::GenesisSpread),
        "expected GenesisSpread (…), got: {:?}", kinds
    );
}

// ── Genesis array literal evaluates to an array ───────────────────────────────

#[test]
fn genesis_array_literal_evaluates() {
    let src = "let arr = ⧉1‥2‥3⧉;\nprint(arr[0]);\nprint(arr[2]);\n";
    let out = run_snippet(src);
    assert!(out.contains('1'), "expected element 1, got: {:?}", out);
    assert!(out.contains('3'), "expected element 3, got: {:?}", out);
}

