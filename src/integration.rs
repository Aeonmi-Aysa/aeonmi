//! Simplified AEONMI integration layer.
//! Provides a single entry point that can spin up the
//! classic shell today while leaving room for future
//! Mother AI or Titan integrations.

#![allow(dead_code)]

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub enable_mother_ai: bool,
    pub enable_titan_libraries: bool,
    pub enable_quantum_vault: bool,
    pub config_path: Option<PathBuf>,
    pub pretty_errors: bool,
    pub skip_sema: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            enable_mother_ai: cfg!(feature = "mother-ai"),
            enable_titan_libraries: cfg!(feature = "titan-libraries"),
            enable_quantum_vault: cfg!(feature = "quantum-vault"),
            config_path: None,
            pretty_errors: true,
            skip_sema: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Unified,
    Shell,
    Compile,
}

pub struct AeonmiSystem {
    pub config: SystemConfig,
    pub mode: ExecutionMode,
}

impl AeonmiSystem {
    pub fn new(config: SystemConfig) -> Self {
        Self {
            config,
            mode: ExecutionMode::Unified,
        }
    }

    pub async fn initialize(config: SystemConfig) -> anyhow::Result<Self> {
        Ok(Self::new(config))
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        match self.mode {
            ExecutionMode::Unified => self.run_unified_mode().await,
            ExecutionMode::Shell => self.run_shell_mode().await,
            ExecutionMode::Compile => self.run_compile_mode().await,
        }
    }

    async fn run_unified_mode(&self) -> anyhow::Result<()> {
        // For now we delegate to the existing shell. As
        // Mother AI and Titan land in Rust we can branch
        // here for richer behaviours.
        self.run_shell_mode().await
    }

    async fn run_shell_mode(&self) -> anyhow::Result<()> {
        crate::shell::start(
            self.config.config_path.clone(),
            self.config.pretty_errors,
            self.config.skip_sema,
        )
    }

    async fn run_compile_mode(&self) -> anyhow::Result<()> {
        // Placeholder for future compile-only workflow.
        Ok(())
    }

    pub fn print_system_status(&self) {
        println!("AEONMI integration status:");
        println!("  Mother AI enabled: {}", self.config.enable_mother_ai);
        println!(
            "  Titan libraries enabled: {}",
            self.config.enable_titan_libraries
        );
        println!(
            "  Quantum vault enabled: {}",
            self.config.enable_quantum_vault
        );
    }

    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        println!("Shutting down AEONMI system.");
        Ok(())
    }
}

pub async fn initialize_aeonmi() -> anyhow::Result<AeonmiSystem> {
    AeonmiSystem::initialize(SystemConfig::default()).await
}

pub async fn initialize_aeonmi_with_config(config: SystemConfig) -> anyhow::Result<AeonmiSystem> {
    AeonmiSystem::initialize(config).await
}

pub async fn start_unified() -> anyhow::Result<()> {
    let mut system = initialize_aeonmi().await?;
    system.mode = ExecutionMode::Unified;
    system.run().await
}

pub async fn start_interactive_shell(config: SystemConfig) -> anyhow::Result<()> {
    let mut system = AeonmiSystem::new(config);
    system.mode = ExecutionMode::Shell;
    system.run().await
}
