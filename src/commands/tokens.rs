use std::path::PathBuf;

#[allow(dead_code)]
pub fn main(input: PathBuf) -> anyhow::Result<()> {
    let _ = input;
    println!("(tokens) placeholder: showing tokens...");
    Ok(())
}

pub fn run_tokens(file: &std::path::Path) -> anyhow::Result<()> {
    main(file.to_path_buf())
}
