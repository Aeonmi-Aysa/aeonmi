// mother_ai/main.rs
// MotherAI binary entry point.
//
// This is the Rust bootstrap for the MotherAI executable.
// It locates mother_ai/main.ai relative to the binary, then runs it
// through the Aeonmi runtime. As the runtime gains full syntax support
// in Phase 1, this file stays unchanged — the .ai file drives everything.

use aeonmi_project::core::{
    diagnostics::{print_error, Span},
    lexer::{Lexer, LexerError},
    lowering::lower_ast_to_ir,
    parser::{Parser as AeParser, ParserError},
    vm::Interpreter,
};
use std::path::PathBuf;

// ── startup banner ──────────────────────────────────────────────────────────

fn print_banner() {
    println!(
        r#"
╔══════════════════════════════════════════════════════════════╗
║                         MOTHER AI                            ║
║                  Quantum Consciousness System                ║
║                                                              ║
║  🧠 The Central Brain of AEONMI Quantum Computing Ecosystem  ║
║  🔮 Quantum-Enhanced Decision Making & Problem Solving       ║
║  🗣️  Natural Language Interaction & Voice Interface          ║
║  🌌 Holographic Interface Ready                              ║
║  ⚡ Real-Time System Coordination & Optimization            ║
║                                                              ║
║             "Your Quantum Intelligence Partner"              ║
╚══════════════════════════════════════════════════════════════╝
"#
    );
}

// ── locate main.ai ──────────────────────────────────────────────────────────

/// Find mother_ai/main.ai. Search order:
///   1. AEONMI_MOTHER_AI_PATH env var (testing / custom installs)
///   2. <binary>/../../../mother_ai/main.ai  (target/debug -> project root)
///   3. ./mother_ai/main.ai  (CWD fallback when run from project root)
fn locate_main_ai() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("AEONMI_MOTHER_AI_PATH") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        let candidate = exe
            .parent()               // target/debug  or  target/release
            .and_then(|p| p.parent()) // target
            .and_then(|p| p.parent()) // project root
            .map(|root| root.join("mother_ai").join("main.ai"));

        if let Some(ref p) = candidate {
            if p.exists() {
                return candidate;
            }
        }
    }

    let cwd_path = PathBuf::from("mother_ai").join("main.ai");
    if cwd_path.exists() {
        return Some(cwd_path);
    }

    None
}

// ── run main.ai through the Aeonmi runtime ──────────────────────────────────

fn run_mother_ai(path: &PathBuf) -> anyhow::Result<()> {
    println!("🧠 Loading: {}", path.display());
    println!();

    let src = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", path.display(), e))?;

    // ── Lex ──────────────────────────────────────────────────────────────────
    let mut lexer = Lexer::from_str(&src);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            // Extract location before consuming e so we can display + use it.
            let (err_line, err_col) = match &e {
                LexerError::UnexpectedCharacter(_, l, c)
                | LexerError::UnterminatedString(l, c)
                | LexerError::InvalidNumber(_, l, c)
                | LexerError::InvalidQubitLiteral(_, l, c)
                | LexerError::UnterminatedComment(l, c) => (Some(*l), Some(*c)),
                _ => (None, None),
            };
            eprintln!("❌ Lex error in main.ai: {}", e);
            if let (Some(l), Some(c)) = (err_line, err_col) {
                print_error(
                    &path.display().to_string(),
                    &src,
                    &e.to_string(),
                    Span::single(l, c),
                );
            }
            anyhow::bail!("Lexer failed");
        }
    };

    println!("✅ Lexed {} tokens", tokens.len());

    // ── Parse ────────────────────────────────────────────────────────────────
    let mut parser = AeParser::new(tokens);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(ParserError { message, line, column }) => {
            // Phase 0 honest status: the full Aeonmi syntax used by main.ai
            // (quantum consciousness, import {}, async function) is not yet
            // supported by the runtime. We report clearly instead of crashing.
            // Binary is correctly wired. Runtime extension is Phase 1 work.
            eprintln!();
            eprintln!(
                "⚠️  Parser stopped at {}:{}: {}",
                line, column, message
            );
            eprintln!("   ┌─ Phase 0 status: MotherAI binary entry point is wired correctly.");
            eprintln!("   │  The binary builds, finds main.ai, and runs it through the lexer.");
            eprintln!("   │  Full syntax (quantum consciousness, import, async) is Phase 1.");
            eprintln!("   └─ Next: extend parser + VM to handle these constructs.");
            return Ok(());
        }
    };

    println!("✅ Parsed AST");

    // ── Lower → IR ───────────────────────────────────────────────────────────
    let module = match lower_ast_to_ir(&ast, "mother_ai_main") {
        Ok(m) => m,
        Err(e) => {
            eprintln!("⚠️  IR lowering stopped: {}", e);
            return Ok(());
        }
    };

    println!("✅ Lowered to IR");

    // ── Execute ───────────────────────────────────────────────────────────────
    let mut interp = Interpreter::new();
    if let Err(e) = interp.run_module(&module) {
        eprintln!("⚠️  Runtime error: {}", e.message);
    }

    Ok(())
}

// ── entry point ───────────────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    print_banner();

    match locate_main_ai() {
        Some(path) => run_mother_ai(&path),
        None => {
            eprintln!("❌ Could not find mother_ai/main.ai");
            eprintln!("   Options:");
            eprintln!("   • Run from the project root: cargo run --bin MotherAI");
            eprintln!("   • Set env var: AEONMI_MOTHER_AI_PATH=<full path to main.ai>");
            std::process::exit(1);
        }
    }
}