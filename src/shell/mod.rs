pub mod repl;

pub use repl::start;

use crate::mother::embryo_loop::{EmbryoConfig, EmbryoLoop};
use anyhow::Result;

pub fn run_shell() -> Result<()> {
    let config = EmbryoConfig::default();
    let embryo = EmbryoLoop::new(config);
    println!("Aeonmi Shell v1.0");
    Ok(())
}