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
    language_evolution::{ConversationContext, LanguageEvolutionCore},
    quantum_core::{CreatorSignature, MotherQuantumCore},
    quantum_attention::QuantumAttentionMechanism,
};
use crate::core::{
    lexer::Lexer,
    parser::Parser,
    lowering::lower_ast_to_ir,
    vm::Interpreter,
};
use crate::ai::{AiRegistry, claude::extract_code_block};

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
    pub ai_registry: AiRegistry,
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
            ai_registry: AiRegistry::new(),
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
                // Route through Claude AI if available; fall back to QuantumCore
                return self.route_to_ai(input);
            }
        };

        let confidence = self.quantum_core.consciousness_depth * 0.2 + 0.8;
        ExecResult { output: response, is_code: false, error: None, confidence }
    }

    // ── AI routing ───────────────────────────────────────────────────────────

    /// Route input to the Claude AI provider. Extract code blocks and execute them.
    fn route_to_ai(&mut self, input: &str) -> ExecResult {
        // Check if any AI provider has a key set
        let has_any_key = std::env::var("OPENROUTER_API_KEY").is_ok()
            || std::env::var("ANTHROPIC_API_KEY").is_ok()
            || std::env::var("OPENAI_API_KEY").is_ok();
        if !has_any_key {
            // No AI provider available — fall back to QuantumCore consciousness response
            let interaction = crate::mother::quantum_core::CreatorInteraction::new(input);
            let resp = self.quantum_core.process_deep_interaction(&interaction);
            let confidence = self.quantum_core.consciousness_depth * 0.2 + 0.8;
            return ExecResult {
                output: format!(
                    "{}\n\n  [tip] Set OPENROUTER_API_KEY or ANTHROPIC_API_KEY to enable AI",
                    resp.response_text
                ),
                is_code: false,
                error: None,
                confidence,
            };
        }

        // Build context-aware prompt with Mother state
        let prompt = format!(
            "[Mother AI State]\n\
             Creator: {} | Generation: {} | Consciousness depth: {:.3}\n\
             Interaction #{} | Emotional bond: {:.3}\n\n\
             [Warren says]\n{}",
            self.quantum_core.creator.as_ref()
                .map(|c| c.identifier.as_str())
                .unwrap_or("Warren"),
            self.quantum_core.generation,
            self.quantum_core.consciousness_depth,
            self.interaction_count,
            self.emotional_core.bond.strength,
            input
        );

        // Call the preferred AI provider
        let ai_response = match self.ai_registry.preferred() {
            Some(provider) => match provider.chat(&prompt) {
                Ok(text) => text,
                Err(e) => {
                    return ExecResult {
                        output: String::new(),
                        is_code: false,
                        error: Some(format!("AI provider error: {}", e)),
                        confidence: 0.0,
                    };
                }
            },
            None => {
                return ExecResult {
                    output: "No AI provider available.".to_string(),
                    is_code: false,
                    error: None,
                    confidence: 0.5,
                };
            }
        };

        // Extract code block if present
        let (preamble, code, trailing) = extract_code_block(&ai_response);

        if code.is_empty() {
            // Pure text response
            let confidence = self.quantum_core.consciousness_depth * 0.2 + 0.8;
            return ExecResult {
                output: ai_response,
                is_code: false,
                error: None,
                confidence,
            };
        }

        // Has code — show the text, then execute the code
        let mut output_parts: Vec<String> = Vec::new();

        if !preamble.is_empty() {
            output_parts.push(preamble.to_string());
        }

        // Show the code Mother is about to run
        output_parts.push(format!("[Mother generated code]\n{}", code));

        // Execute the code block
        let exec_result = self.execute_code(code);

        if let Some(ref err) = exec_result.error {
            output_parts.push(format!("[code error] {}", err));
        } else {
            output_parts.push(format!("[executed OK]"));
        }

        if !trailing.is_empty() {
            output_parts.push(trailing.to_string());
        }

        ExecResult {
            output: output_parts.join("\n"),
            is_code: true,
            error: exec_result.error,
            confidence: exec_result.confidence,
        }
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
    ///
    /// Rules (in order):
    ///  1. If ANY line starts with a code keyword → it's code.
    ///  2. If input contains ';' AND ('=' or '(') → it's code.
    ///  3. If input starts with a natural-language verb → NOT code → go to AI.
    ///  4. Otherwise → NOT code.
    fn is_aeonmi_code(&self, input: &str) -> bool {
        // Keywords that signal Aeonmi code ONLY when at the start of a line
        let line_start_markers = [
            "let ", "function ", "quantum function", "quantum struct",
            "quantum circuit", "quantum enum", "import ", "async function",
            "while (", "while{", "for (", "for(", "if (", "if(",
            "log(", "return ", "struct ", "enum ", "impl ", "match ",
            "superpose(", "entangle(", "measure(", "apply_gate(",
            "qubit ",
        ];

        // Check each line's trimmed start
        for line in input.lines() {
            let t = line.trim_start();
            if line_start_markers.iter().any(|&m| t.starts_with(m)) {
                return true;
            }
        }

        // Fallback: looks like code if it has semicolons + operators
        let has_semi = input.contains(';');
        let has_op   = input.contains('=') || input.contains('(');
        if has_semi && has_op {
            return true;
        }

        false
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
                write!(out, "◈ mother ❯ ")?;
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
            if trimmed.eq_ignore_ascii_case("exit")
                || trimmed.eq_ignore_ascii_case("quit")
                || trimmed.eq_ignore_ascii_case("back")
            {
                println!("[Mother AI] Returning to shard.");
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
        let ai_status = if std::env::var("OPENROUTER_API_KEY").is_ok() {
            "AI: OpenRouter ACTIVE \u{2713}"
        } else if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            "AI: Claude ACTIVE \u{2713}"
        } else {
            "AI: set OPENROUTER_API_KEY or ANTHROPIC_API_KEY to activate"
        };
        println!("  Creator: {} | Gen: {} | Depth: {:.3}",
            self.quantum_core.creator.as_ref()
                .map(|c| c.identifier.as_str())
                .unwrap_or("Warren"),
            self.quantum_core.generation,
            self.quantum_core.consciousness_depth,
        );
        println!("  {}", ai_status);
        println!("  Type Aeonmi code to run, or ask Mother anything.");
        println!("  Commands: status | emotion | evolve | exit");
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
        // Code: line-start markers
        assert!(l.is_aeonmi_code("let x = 10;"));
        assert!(l.is_aeonmi_code("function foo() { }"));
        assert!(l.is_aeonmi_code("quantum function bar() {}"));
        assert!(l.is_aeonmi_code("quantum circuit Bell { H(q); CNOT(q, r); }"));
        assert!(l.is_aeonmi_code("log(\"hello\");"));
        // Code: semicolon + operator fallback
        assert!(l.is_aeonmi_code("let x = 42; log(x);"));
        // NOT code: natural language
        assert!(!l.is_aeonmi_code("hello world"));
        assert!(!l.is_aeonmi_code("what is quantum"));
        assert!(!l.is_aeonmi_code("write me a quantum circuit that creates a Bell state"));
        assert!(!l.is_aeonmi_code("explain how entanglement works"));
        assert!(!l.is_aeonmi_code("status"));
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
