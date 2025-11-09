//! Embedded Web Editor for Aeonmi
//!
//! This module provides a self-contained web-based IDE that runs entirely
//! within the Aeonmi binary. It eliminates external dependencies like Node.js
//! and provides a unified development experience.

pub mod server;

pub use server::{start_editor_server, AppState, EditorEvent};

use anyhow::Result;
use std::path::PathBuf;

/// Launch the integrated editor server
pub async fn launch_editor(port: Option<u16>, workspace: Option<PathBuf>) -> Result<()> {
    let port = port.unwrap_or(4000);
    let workspace =
        workspace.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Ensure workspace directory exists
    if !workspace.exists() {
        std::fs::create_dir_all(&workspace)?;
    }

    println!("🎯 Launching Aeonmi Integrated Editor");
    println!("📁 Workspace: {}", workspace.display());

    server::start_editor_server(port, workspace).await
}

/// Check if the editor should be launched based on command line arguments
pub fn should_launch_editor(args: &[String]) -> bool {
    // Only launch editor if explicitly requested with --editor flag
    // No longer auto-launch on empty arguments - default to CLI mode

    args.iter().any(|arg| arg == "--editor" || arg == "editor")
}

/// Extract editor-specific arguments
pub fn parse_editor_args(args: &[String]) -> (Option<u16>, Option<PathBuf>) {
    let mut port = None;
    let mut workspace = None;

    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--port" | "-p" => {
                if let Some(port_str) = args.get(i + 1) {
                    if let Ok(p) = port_str.parse::<u16>() {
                        port = Some(p);
                    }
                }
            }
            "--workspace" | "-w" => {
                if let Some(workspace_str) = args.get(i + 1) {
                    workspace = Some(PathBuf::from(workspace_str));
                }
            }
            _ => {}
        }
    }

    (port, workspace)
}
