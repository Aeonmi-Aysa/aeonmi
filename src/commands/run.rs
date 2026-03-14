use colored::Colorize;
use std::path::PathBuf;

// Native interpreter pieces
use crate::core::lexer::Lexer;
use crate::core::parser::{Parser as AeParser, ParserError};
use crate::core::lowering::lower_ast_to_ir;
use crate::core::vm::Interpreter;
use crate::core::diagnostics::{print_error, emit_json_error, Span};
use crate::core::lexer::LexerError;

/// Public native interpreter entry (no JS emission)
pub fn run_native(
    input: &PathBuf,
    pretty: bool,
    no_sema: bool,
) -> anyhow::Result<()> {
    println!("native: executing {}", input.display());
    let source = std::fs::read_to_string(input)?;
    // Lex
    let mut lexer = Lexer::from_str(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            if pretty {
                match e {
                    LexerError::UnexpectedCharacter(_, line, col)
                    | LexerError::UnterminatedString(line, col)
                    | LexerError::InvalidNumber(_, line, col)
                    | LexerError::InvalidQubitLiteral(_, line, col)
                    | LexerError::UnterminatedComment(line, col) => {
                        emit_json_error(
                            &input.display().to_string(),
                            &format!("{}", e),
                            &Span::single(line, col),
                        );
                        print_error(
                            &input.display().to_string(),
                            &source,
                            &format!("{}", e),
                            Span::single(line, col),
                        );
                    }
                    _ => eprintln!("{} Lexing error: {}", "error:".bright_red(), e),
                }
            } else {
                eprintln!("{} Lexing error: {}", "error:".bright_red(), e);
            }
            return Ok(()); // mimic compile path exit(1) without aborting process
        }
    };
    // Parse
    let mut parser = AeParser::new(tokens.clone());
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(ParserError { message, line, column }) => {
            if pretty {
                emit_json_error(
                    &input.display().to_string(),
                    &format!("Parsing error: {}", message),
                    &Span::single(line, column),
                );
                print_error(
                    &input.display().to_string(),
                    &source,
                    &format!("Parsing error: {}", message),
                    Span::single(line, column),
                );
            } else {
                eprintln!("{} Parsing error: {}", "error:".bright_red(), message);
            }
            return Ok(());
        }
    };
    if no_sema {
        println!("note: semantic analysis skipped (native)");
    }
    // Lower & interpret
    match lower_ast_to_ir(&ast, "main") {
        Ok(module) => {
            let mut interp = Interpreter::new();
            // Set base_dir so import resolution works relative to the source file
            interp.base_dir = input.parent().map(|p| p.to_path_buf());
            if let Err(e) = interp.run_module(&module) {
                eprintln!("{} runtime error: {}", "error:".bright_red(), e.message);
            }
        }
        Err(e) => eprintln!("{} lowering error: {}", "error:".bright_red(), e),
    }
    Ok(())
}

pub fn main_with_opts(
    input: PathBuf,
    _out: Option<PathBuf>,
    pretty: bool,
    no_sema: bool,
) -> anyhow::Result<()> {
    // Shard always runs natively — no Node.js, no .js temp files.
    run_native(&input, pretty, no_sema)
}
