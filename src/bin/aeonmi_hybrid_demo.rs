use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use aeonmi_project::commands::run::run_native;
use aeonmi_project::core::ast::ASTNode;
use aeonmi_project::core::lexer::{Lexer, LexerError};
use aeonmi_project::core::parser::{Parser as AeParser, ParserError};
use aeonmi_project::core::token::Token;
use anyhow::{anyhow, Context, Result};
use tempfile::NamedTempFile;

const SAMPLE_CODE: &str = include_str!("../../examples/aeonmi_hybrid_showcase.ai");

fn main() -> Result<()> {
    println!("========================================");
    println!("      Aeonmi Hybrid Interactive Demo");
    println!("========================================\n");
    println!("This standalone command-line app proves the Aeonmi language");
    println!("toolchain is live by running the native lexer, parser, and");
    println!("interpreter against a real .ai program.\n");

    loop {
        println!("Choose an option:");
        println!(" 1) View the embedded Aeonmi sample");
        println!(" 2) Lex the sample and display raw tokens");
        println!(" 3) Parse the sample and inspect the AST");
        println!(" 4) Execute the sample with the native interpreter");
        println!(" 5) Paste your own Aeonmi snippet for analysis");
        println!(" 6) Export the sample to ./aeonmi_hybrid_sample.ai");
        println!(" q) Quit");
        print!("> ");
        io::stdout().flush()?;

        let mut selection = String::new();
        io::stdin().read_line(&mut selection)?;
        let choice = selection.trim();

        match choice {
            "1" => show_sample(),
            "2" => display_tokens(SAMPLE_CODE),
            "3" => display_ast(SAMPLE_CODE),
            "4" => run_sample()?,
            "5" => analyze_user_snippet()?,
            "6" => export_sample()?,
            "q" | "Q" => {
                println!("Goodbye.");
                break;
            }
            _ => {
                println!("Unknown option. Please choose 1-6 or q.\n");
            }
        }
    }

    Ok(())
}

fn show_sample() {
    println!("\n--- Embedded Aeonmi Sample -------------------------------");
    println!("{}", SAMPLE_CODE);
    println!("---------------------------------------------------------\n");
}

fn display_tokens(source: &str) {
    match lex_source(source) {
        Ok(tokens) => {
            println!("\nToken stream ({} tokens):", tokens.len());
            for token in tokens {
                println!("  {:?}", token);
            }
            println!();
        }
        Err(err) => {
            println!("Lexer error: {}\n", err);
        }
    }
}

fn display_ast(source: &str) {
    match parse_source(source) {
        Ok(ast) => {
            println!("\nAST summary: total nodes = {}", count_nodes(&ast));
            println!("{:#?}\n", ast);
        }
        Err(err) => {
            println!("Parser error: {}\n", err);
        }
    }
}

fn run_sample() -> Result<()> {
    println!("\nRunning Aeonmi sample...\n");

    let mut temp_file = NamedTempFile::new().context("failed to create temp file")?;
    temp_file
        .write_all(SAMPLE_CODE.as_bytes())
        .context("failed to write sample code")?;
    temp_file.flush()?;

    let path: PathBuf = temp_file.path().to_path_buf();
    run_native(&path, false, false).context("failed to execute sample")?;
    println!();

    Ok(())
}

fn analyze_user_snippet() -> Result<()> {
    println!("\nPaste Aeonmi code (finish with an empty line):");
    let mut buffer = String::new();
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        if line.trim().is_empty() {
            break;
        }
        buffer.push_str(&line);
    }

    if buffer.trim().is_empty() {
        println!("No code entered. Returning to menu.\n");
        return Ok(());
    }

    println!("\nLexing snippet...");
    let tokens = match lex_source(&buffer) {
        Ok(tokens) => tokens,
        Err(err) => {
            println!("Lexer error: {}\n", err);
            return Ok(());
        }
    };
    println!("Token count: {}", tokens.len());

    println!("Parsing snippet...");
    match parse_tokens(tokens) {
        Ok(ast) => {
            println!("AST node count: {}", count_nodes(&ast));
            println!("{:#?}\n", ast);
        }
        Err(err) => {
            println!("Parser error: {}\n", err);
        }
    }

    Ok(())
}

fn export_sample() -> Result<()> {
    let path = PathBuf::from("aeonmi_hybrid_sample.ai");
    if path.exists() {
        println!("Sample already exists at {:?}.", path);
        return Ok(());
    }

    fs::write(&path, SAMPLE_CODE).with_context(|| format!("failed to write {:?}", path))?;
    println!("Sample written to {:?}.\n", path);
    Ok(())
}

fn lex_source(source: &str) -> Result<Vec<Token>> {
    let mut lexer = Lexer::from_str(source);
    lexer
        .tokenize()
        .map_err(|err| anyhow!("{}", format_lexer_error(err)))
}

fn parse_source(source: &str) -> Result<ASTNode> {
    let tokens = lex_source(source)?;
    parse_tokens(tokens)
}

fn parse_tokens(tokens: Vec<Token>) -> Result<ASTNode> {
    let mut parser = AeParser::new(tokens);
    parser
        .parse()
        .map_err(|err| anyhow!("{}", format_parser_error(err)))
}

fn count_nodes(node: &ASTNode) -> usize {
    match node {
        ASTNode::Program(nodes)
        | ASTNode::Block(nodes) => 1 + nodes.iter().map(count_nodes).sum::<usize>(),
        ASTNode::Function { body, .. } => 1 + body.iter().map(count_nodes).sum::<usize>(),
        ASTNode::For { body, init, condition, increment } => {
            let mut total = 1;
            if let Some(init) = init {
                total += count_nodes(init);
            }
            if let Some(cond) = condition {
                total += count_nodes(cond);
            }
            if let Some(inc) = increment {
                total += count_nodes(inc);
            }
            total + count_nodes(body)
        }
        ASTNode::While { condition, body } => 1 + count_nodes(condition) + count_nodes(body),
        ASTNode::If { condition, then_branch, else_branch } => {
            let mut total = 1 + count_nodes(condition) + count_nodes(then_branch);
            if let Some(else_branch) = else_branch {
                total += count_nodes(else_branch);
            }
            total
        }
        ASTNode::Assignment { value, .. }
        | ASTNode::Return(value)
        | ASTNode::Log(value)
        | ASTNode::UnaryExpr { value, .. }
        | ASTNode::Group(value)
        | ASTNode::Call { callee: value, args: _ }
        | ASTNode::Index { target: value, index: _ }
        | ASTNode::QubitDecl(value)
        | ASTNode::QuantumCall { target: value, .. }
        | ASTNode::QuantumMeasure { target: value, .. }
        | ASTNode::HieroglyphicCall { target: value, .. }
        | ASTNode::Await(value) => 1 + count_nodes(value),
        ASTNode::BinaryExpr { left, right, .. }
        | ASTNode::Logical { left, right, .. }
        | ASTNode::Comparison { left, right, .. } => 1 + count_nodes(left) + count_nodes(right),
        ASTNode::Call { args, .. } => 1 + args.iter().map(count_nodes).sum::<usize>(),
        ASTNode::ArrayLiteral(items)
        | ASTNode::TupleLiteral(items)
        | ASTNode::StructLiteral { fields: items, .. }
        | ASTNode::IntrinsicArgs(items) => 1 + items.iter().map(count_nodes).sum::<usize>(),
        ASTNode::VariableDecl { value, .. }
        | ASTNode::Let { value, .. }
        | ASTNode::Const { value, .. } => 1 + count_nodes(value),
        ASTNode::Break
        | ASTNode::Continue
        | ASTNode::Identifier { .. }
        | ASTNode::NumberLiteral { .. }
        | ASTNode::StringLiteral { .. }
        | ASTNode::BooleanLiteral { .. }
        | ASTNode::NullLiteral
        | ASTNode::QubitLiteral { .. }
        | ASTNode::HieroglyphicLiteral { .. }
        | ASTNode::Error => 1,
    }
}

fn format_lexer_error(err: LexerError) -> String {
    format!("{}", err)
}

fn format_parser_error(err: ParserError) -> String {
    format!("{}", err)
}
