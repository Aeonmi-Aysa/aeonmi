use crate::mother::embryo_loop::{EmbryoConfig, EmbryoLoop};

pub fn main() -> anyhow::Result<()> {
    let config = EmbryoConfig {
        interactive: true,
        verbose: false,
        ..Default::default()
    };
    let mut mother = EmbryoLoop::new(config);
    mother.run_repl()
}
