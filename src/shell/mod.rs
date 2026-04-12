pub mod repl;

use crate::mother::embryo_loop::{EmbryoConfig, EmbryoLoop};
use anyhow::Result;
use std::io::{self, Write};

pub fn run_shell() -> Result<()> {
    let config = EmbryoConfig::default();
    let mut embryo = EmbryoLoop::new(config);
    
    println!("Aeonmi Shell v1.0");
    println!("Type 'exit' to quit/n");
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let trimmed = input.trim();
        if trimmed == "exit" {
            break;
        }
        
        println!("Echo: {}", trimmed);
    }
    
    Ok(())
}