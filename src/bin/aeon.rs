// Aeon CLI binary - forwards to main Aeonmi implementation
// This creates the "aeon" command that users interact with

use anyhow::Result;

fn main() -> Result<()> {
    // Simply forward to the enhanced CLI implementation
    std::env::set_var("AEON_ENHANCED_CLI", "1");
    aeonmi_project::cli_integration::run_enhanced_aeon_cli()
}
