pub fn main() -> anyhow::Result<()> {
    println!("(repl) placeholder: starting REPL...");
    Ok(())
}

pub fn start_repl(backend: &str, load: Option<&std::path::Path>) -> anyhow::Result<()> {
    let _ = backend;
    let _ = load;
    main()
}
