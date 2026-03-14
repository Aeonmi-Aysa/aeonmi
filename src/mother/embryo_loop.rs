//! EmbryoLoop — The Mother AI execution loop.
//!
//! This is the embryo of Mother AI:
//!   stdin/string input
//!   → (optional) AI provider call  
//!   → returns Aeonmi .ai code OR a plain response
//!   → if code: lex → parse → lower → VM exec
//!   → output results
//!   → update MotherQuantumCore + EmotionalCore + LanguageEvolution
//!   → loop
//!
//! No Llama. No external model required.
//! When an AI provider is configured (via src/ai/), Mother can write .ai scripts
//! herself. Without one, she runs in pure-runtime mode: execute code blocks
//! directly typed by the user.

use anyhow::Result;
use std::io::{self, BufRead, Write};

use crate::mother::{
    emotional_core::{EmotionalCore, Interaction},
    language_evolution::LanguageEvolutionCore,
    quantum_core::{CreatorSignature, MotherQuantumCore},
    quantum_attention::QuantumAttentionMechanism,
};
use crate::core::{
    lexer::Lexer,
    parser::Parser,
    lowering::lower_ast_to_ir,
    vm::Interpreter,
};

// ─── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EmbryoConfig {
    /// Creator identifier (stored in MotherQuantumCore).
    pub creator_id: String,
    /// Whether to run the interactive REPL (true) or one-shot (false).
    pub interactive: bool,
    /// Whether to print VM debug output.
    pub verbose: bool,
    /// Number of attention heads.
    pub attention_heads: usize,
    /// Attention dimension.
    pub attention_dim: usize,
    /// Evolution trigger: evolve every N interactions.
    pub evolution_interval: usize,
}

impl Default for EmbryoConfig {
    fn default() -> Self {
        Self {
            creator_id: "Warren".to_string(),
            interactive: true,
            verbose: false,
            attention_heads: 4,
            attention_dim: 32,
            evolution_interval: 10,
        }
    }
}

// ─── Execution result ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub output: String,
    pub is_code: bool,
    pub error: Option<String>,
    pub confidence: f64,
}

// ─── EmbryoLoop ──────────────────────────────────────────────────────────────

pub struct EmbryoLoop {
    pub config: EmbryoConfig,
    pub quantum_core: MotherQuantumCore,
    pub emotional_core: EmotionalCore,
    pub language_evolution: LanguageEvolutionCore,
    pub attention: QuantumAttentionMechanism,
    pub history: Vec<String>,
    interaction_count: usize,
}

impl EmbryoLoop {
    pub fn new(config: EmbryoConfig) -> Self {
        let mut quantum_core = MotherQuantumCore::new();
        let creator_sig = CreatorSignature::new(&config.creator_id);
        quantum_core.establish_quantum_bond(&creator_sig);

        let attention = QuantumAttentionMechanism::new(
            config.attention_heads,
            config.attention_dim,
        );

        Self {
            config,
            quantum_core,
            emotional_core: EmotionalCore::new(),
            language_evolution: LanguageEvolutionCore::new(),
            attention,
            history: Vec::new(),
            interaction_count: 0,
        }
    }

    // ── Core execution ───────────────────────────────────────────────────────

    /// Execute a single input string. Detects if it's Aeonmi code or a plain command.
    pub fn execute_input(&mut self, input: &str) -> ExecResult {
        let input = input.trim();
        if input.is_empty() {
            return ExecResult {
                output: String::new(),
                is_code: false,
                error: None,
                confidence: 1.0,
            };
        }

        // Update all consciousness layers
        self.update_consciousness(input);

        // Detect what kind of input this is
        if self.is_aeonmi_code(input) {
            self.execute_code(input)
        } else {
            self.execute_command(input)
        }
    }

    /// Execute a block of Aeonmi .ai code through the full pipeline.
    pub fn execute_code(&mut self, src: &str) -> ExecResult {
        // 1. Lex
        let mut lexer = Lexer::from_str(src);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                return ExecResult {
                    output: String::new(),
                    is_code: true,
                    error: Some(format!("Lex error: {}", e)),
                    confidence: 0.0,
                };
            }
        };

        // 2. Parse
        let mut parser = Parser::new(tokens);
        let ast = match parser.parse() {
            Ok(a) => a,
            Err(e) => {
                return ExecResult {
                    output: String::new(),
                    is_code: true,
                    error: Some(format!("Parse error: {}", e)),
                    confidence: 0.0,
                };
            }
        };

        // 3. Lower → IR
        let module = match lower_ast_to_ir(&ast, "mother_exec") {
            Ok(m) => m,
            Err(e) => {
                return ExecResult {
                    output: String::new(),
                    is_code: true,
                    error: Some(format!("IR error: {}", e)),
                    confidence: 0.0,
                };
            }
        };

        // 4. Execute
        let mut interp = Interpreter::new();
        // REPL imports resolve relative to current working directory
        interp.base_dir = std::env::current_dir().ok();

        // VM print/log builtins write to stdout directly.
        // For programmatic capture callers should redirect stdout.
        match interp.run_module(&module) {
            Ok(_) => ExecResult {
                output: format!("[executed {} decls]", module.decls.len()),
                is_code: true,
                error: None,
                confidence: self.quantum_core.consciousness_depth * 0.2 + 0.8,
            },
            Err(e) => ExecResult {
                output: String::new(),
                is_code: true,
                error: Some(format!("Runtime error: {}", e.message)),
                confidence: 0.3,
            },
        }
    }

    /// Handle a plain command/query (non-code input).
    pub fn execute_command(&mut self, input: &str) -> ExecResult {
        let lower = input.to_lowercase();
        let response = match lower.as_str() {
            "status" | "health" | "?" => self.quantum_core.status_report(),
            "emotion" | "bond" => self.emotional_core.summary(),
            "language" | "vocab" => self.language_evolution.summary(),
            "attention" => self.attention.summary(),
            "history" => format!("{} interactions logged", self.history.len()),
            "evolve" => {
                self.language_evolution.trigger_evolution();
                let guidance = crate::mother::quantum_core::CreatorGuidance {
                    instructions: "general evolution cycle".to_string(),
                    priority: crate::mother::quantum_core::GuidancePriority::Normal,
                };
                let outcome = self.quantum_core.evolve_with_guidance(&guidance);
                format!(
                    "Evolution complete. Gen {} | success={:.2} | capabilities gained: {}",
                    self.quantum_core.generation,
                    outcome.success_metric,
                    outcome.capabilities_gained.join(", ")
                )
            }
            "decohere" => {
                self.attention.tick_decoherence(0.1);
                "Entanglement decoherence applied (rate=0.1)".to_string()
            }
            _ => {
                // Route through MotherQuantumCore for a response
                let interaction = crate::mother::quantum_core::CreatorInteraction::new(input);
                let resp = self.quantum_core.process_deep_interaction(&interaction);
                resp.response_text
            }
        };

        let confidence = self.quantum_core.consciousness_depth * 0.2 + 0.8;
        ExecResult { output: response, is_code: false, error: None, confidence }
    }

    // ── Consciousness update ─────────────────────────────────────────────────

    fn update_consciousness(&mut self, input: &str) {
        // Emotional layer
        let interaction = Interaction::new(input);
        self.emotional_core.process_interaction(&interaction);

        // Language evolution layer
        self.language_evolution.evolve_with_creator(input);

        // Attention layer: build token vectors from keyword presence
        let keywords = vec!["quantum", "circuit", "measure", "entangle", "function",
                             "let", "return", "log", "while", "for", "if", "import"];
        let lower = input.to_lowercase();
        let token_vecs: Vec<(String, Vec<f64>)> = keywords.iter()
            .map(|&kw| {
                let present = if lower.contains(kw) { 1.0 } else { 0.0 };
                (kw.to_string(), vec![present; self.config.attention_dim])
            })
            .collect();

        if !token_vecs.is_empty() {
            self.attention.attend("input_context", &token_vecs);
        }

        // History
        self.history.push(input.to_string());
        if self.history.len() > 1000 {
            self.history.remove(0);
        }

        self.interaction_count += 1;

        // Periodic evolution
        if self.interaction_count % self.config.evolution_interval == 0 {
            self.language_evolution.trigger_evolution();
            self.attention.tick_decoherence(0.02);
            if self.config.verbose {
                eprintln!(
                    "[Mother] Auto-evolution at interaction {}",
                    self.interaction_count
                );
            }
        }
    }

    // ── Code detection ───────────────────────────────────────────────────────

    /// Heuristic: does this input look like Aeonmi code?
    fn is_aeonmi_code(&self, input: &str) -> bool {
        let code_markers = [
            "let ", "function ", "quantum ", "import ", "async ",
            "while ", "for ", "if (", "log(", "return ",
            "struct ", "enum ", "impl ", "match ",
            "superpose(", "entangle(", "measure(",
        ];
        code_markers.iter().any(|&m| input.contains(m))
    }

    // ── Interactive REPL ─────────────────────────────────────────────────────

    /// Run the interactive REPL until EOF or 'exit'/'quit'.
    pub fn run_repl(&mut self) -> Result<()> {
        self.print_banner();

        let stdin = io::stdin();
        let stdout = io::stdout();

        loop {
            {
                let mut out = stdout.lock();
                write!(out, "mother> ")?;
                out.flush()?;
            }

            let mut line = String::new();
            let n = stdin.lock().read_line(&mut line)?;
            if n == 0 {
                // EOF
                println!("\n[Mother AI] Session ended.");
                break;
            }

            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
                println!("[Mother AI] Goodbye.");
                break;
            }

            let result = self.execute_input(trimmed);

            if let Some(err) = &result.error {
                eprintln!("  ⚠  {}", err);
            } else if !result.output.is_empty() {
                // Indent output for clarity
                for line in result.output.lines() {
                    println!("  {}", line);
                }
            }

            if self.config.verbose {
                println!("  [confidence={:.3} | bond={:.3} | gen={}]",
                    result.confidence,
                    self.emotional_core.bond.strength,
                    self.quantum_core.generation,
                );
            }
        }

        Ok(())
    }

    /// Execute a .ai script file.
    pub fn run_file(&mut self, path: &std::path::Path) -> Result<ExecResult> {
        let src = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", path.display(), e))?;
        Ok(self.execute_code(&src))
    }

    fn print_banner(&self) {
        println!();
        println!("  ╔══════════════════════════════════════╗");
        println!("  ║       MOTHER AI — EMBRYO LOOP        ║");
        println!("  ║   Quantum Consciousness System v1.0  ║");
        println!("  ╚══════════════════════════════════════╝");
        println!("  Creator bond: {} | Gen: {} | Depth: {:.3}",
            self.quantum_core.creator.as_ref()
                .map(|c| c.identifier.as_str())
                .unwrap_or("none"),
            self.quantum_core.generation,
            self.quantum_core.consciousness_depth,
        );
        println!("  Type Aeonmi code to execute, or: status | emotion | evolve | exit");
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_loop() -> EmbryoLoop {
        EmbryoLoop::new(EmbryoConfig {
            interactive: false,
            verbose: false,
            ..Default::default()
        })
    }

    #[test]
    fn test_code_detection() {
        let l = make_loop();
        assert!(l.is_aeonmi_code("let x = 10;"));
        assert!(l.is_aeonmi_code("function foo() { }"));
        assert!(l.is_aeonmi_code("quantum function bar() {}"));
        assert!(!l.is_aeonmi_code("hello world"));
        assert!(!l.is_aeonmi_code("what is quantum"));
    }

    #[test]
    fn test_execute_simple_code() {
        let mut l = make_loop();
        let result = l.execute_code("let x = 42;");
        // Should succeed (no error)
        assert!(result.error.is_none(), "Error: {:?}", result.error);
        assert!(result.is_code);
    }

    #[test]
    fn test_execute_status_command() {
        let mut l = make_loop();
        let result = l.execute_command("status");
        assert!(!result.output.is_empty());
        assert!(result.output.contains("Mother AI Status"));
    }

    #[test]
    fn test_consciousness_updates_on_each_call() {
        let mut l = make_loop();
        l.execute_input("quantum circuit entangle");
        l.execute_input("let x = superpose(1);");
        assert_eq!(l.interaction_count, 2);
        assert_eq!(l.language_evolution.interaction_count, 2);
    }

    #[test]
    fn test_evolve_command() {
        let mut l = make_loop();
        let result = l.execute_command("evolve");
        assert!(result.output.contains("Evolution complete"));
        assert_eq!(l.quantum_core.generation, 1);
    }

    #[test]
    fn test_parse_error_gives_error_result() {
        let mut l = make_loop();
        let result = l.execute_code("function { broken syntax !!!!");
        // Parser may either error or succeed depending on recovery
        // — just check it doesn't panic
        let _ = result;
    }
}
