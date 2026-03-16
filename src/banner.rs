//! Cyberpunk ASCII banner for Aeonmi CLI startup.
//! Colors: neon yellow for the language name, magenta for quantum markers.
//! Printed to stderr so it does not pollute captured stdout in tests/pipes.

/// Print the Aeonmi cyberpunk startup banner to stderr.
///
/// The banner is skipped when:
/// - `NO_COLOR` environment variable is set, or
/// - stderr is not a TTY (piped / scripted use).
pub fn print_banner() {
    // Skip in non-interactive contexts.
    if std::env::var("NO_COLOR").is_ok() {
        return;
    }
    #[cfg(unix)]
    if !is_tty() {
        return;
    }

    // ANSI color codes.
    const RESET: &str = "\x1b[0m";
    const NEON_YELLOW: &str = "\x1b[93m";   // bright yellow — Aeonmi brand
    const MAGENTA: &str = "\x1b[95m";        // bright magenta — quantum
    const CYAN: &str = "\x1b[96m";           // bright cyan — accents
    const DIM: &str = "\x1b[2m";             // dim — secondary text

    eprintln!();
    eprintln!(
        "{MAGENTA}  ╔══════════════════════════════════════════════════════╗{RESET}"
    );
    eprintln!(
        "{MAGENTA}  ║{RESET}  {NEON_YELLOW}   █████╗ ███████╗ ██████╗ ███╗   ██╗███╗   ███╗██╗{RESET}  {MAGENTA}║{RESET}"
    );
    eprintln!(
        "{MAGENTA}  ║{RESET}  {NEON_YELLOW}  ██╔══██╗██╔════╝██╔═══██╗████╗  ██║████╗ ████║██║{RESET}  {MAGENTA}║{RESET}"
    );
    eprintln!(
        "{MAGENTA}  ║{RESET}  {NEON_YELLOW}  ███████║█████╗  ██║   ██║██╔██╗ ██║██╔████╔██║██║{RESET}  {MAGENTA}║{RESET}"
    );
    eprintln!(
        "{MAGENTA}  ║{RESET}  {NEON_YELLOW}  ██╔══██║██╔══╝  ██║   ██║██║╚██╗██║██║╚██╔╝██║██║{RESET}  {MAGENTA}║{RESET}"
    );
    eprintln!(
        "{MAGENTA}  ║{RESET}  {NEON_YELLOW}  ██║  ██║███████╗╚██████╔╝██║ ╚████║██║ ╚═╝ ██║██║{RESET}  {MAGENTA}║{RESET}"
    );
    eprintln!(
        "{MAGENTA}  ║{RESET}  {NEON_YELLOW}  ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝     ╚═╝╚═╝{RESET}  {MAGENTA}║{RESET}"
    );
    eprintln!(
        "{MAGENTA}  ╠══════════════════════════════════════════════════════╣{RESET}"
    );
    eprintln!(
        "{MAGENTA}  ║{RESET}  {CYAN}⊗{RESET} Quantum-Native · Self-Hosting · Glyph-Sovereign  {MAGENTA}║{RESET}"
    );
    eprintln!(
        "{MAGENTA}  ║{RESET}  {DIM}  v{} — built on the Shard                          {RESET}{MAGENTA}║{RESET}",
        env!("CARGO_PKG_VERSION")
    );
    eprintln!(
        "{MAGENTA}  ╚══════════════════════════════════════════════════════╝{RESET}"
    );
    eprintln!();
}

/// Check if stderr is a TTY (Unix only).
#[cfg(unix)]
fn is_tty() -> bool {
    use std::os::unix::io::AsRawFd;
    // SAFETY: isatty is a safe POSIX call.
    unsafe { libc::isatty(std::io::stderr().as_raw_fd()) != 0 }
}
