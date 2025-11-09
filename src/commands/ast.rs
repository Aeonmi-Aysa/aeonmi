use std::path::PathBuf;

#[allow(dead_code)]
pub fn main(input: PathBuf) -> anyhow::Result<()> {
    let _ = input;
    println!("(ast) placeholder: showing AST...");
    Ok(())
}

pub fn run_ast(file: &std::path::Path, format: &str) -> anyhow::Result<()> {
    let _ = format;
    main(file.to_path_buf())
}
