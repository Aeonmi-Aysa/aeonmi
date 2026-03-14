//! Aeonmi startup banner — cyberpunk aesthetic.
//! P4-13/P4-14/P4-15: CLI color scheme, neon output, glyph art.

use colored::Colorize;

// Cyberpunk color palette
// Neon yellow  : (255, 240,   0)  — Aeonmi output / identifiers
// Electric cyan: (  0, 255, 255)  — accent borders
// Magenta      : (225,   0, 180)  — quantum operations
// Purple       : (130,   0, 200)  — secondary / paths
// Hot green    : (  0, 255, 150)  — success / harmonized
// Bright red   : (255,  50,  50)  — errors / anomalies

/// Print the full Aeonmi startup banner.
/// Called once on every CLI invocation so users always see the identity.
pub fn print_startup_banner() {
    let cyan    = |s: &str| s.truecolor(0, 255, 255).bold().to_string();
    let yellow  = |s: &str| s.truecolor(255, 240, 0).bold().to_string();
    let magenta = |s: &str| s.truecolor(225, 0, 180).bold().to_string();
    let purple  = |s: &str| s.truecolor(130, 0, 200).to_string();
    let green   = |s: &str| s.truecolor(0, 255, 150).to_string();

    println!();
    println!("  {}",  cyan("╔══════════════════════════════════════════════════════════╗"));
    println!("  {}{}{}",
        cyan("║"),
        yellow("          ▲  A E O N M I   S H A R D  ▲                   "),
        cyan("║"),
    );
    println!("  {}{}{}",
        cyan("║"),
        magenta("     ⊗  Quantum Language Runtime · v1.0 · Sovereign AI  ⊗    "),
        cyan("║"),
    );
    println!("  {}{}{}",
        cyan("║"),
        purple("         ◈  Lexer→Parser→IR→Native VM  ·  Zero-Node.js  ◈     "),
        cyan("║"),
    );
    println!("  {}",  cyan("╚══════════════════════════════════════════════════════════╝"));
    println!("  {}  {}",
        green("✓ quantum · glyph · vault · qube · mother · web3"),
        purple("type 'help' or 'aeonmi --help'"),
    );
    println!();
}

/// One-line version banner for subcommand headers — compact but styled.
pub fn print_command_banner() {
    let yellow  = |s: &str| s.truecolor(255, 240, 0).bold().to_string();
    let magenta = |s: &str| s.truecolor(225, 0, 180).to_string();
    print!("  {} {} ",
        yellow("⟦AEONMI⟧"),
        magenta("⊗"),
    );
}

/// Format a success message in neon green.
pub fn success(msg: &str) -> String {
    format!("  {} {}", "✓".truecolor(0, 255, 150).bold(), msg.truecolor(0, 255, 150))
}

/// Format an error message in bright red.
pub fn error(msg: &str) -> String {
    format!("  {} {}", "✗".truecolor(255, 50, 50).bold(), msg.truecolor(255, 50, 50))
}

/// Format a quantum operation label in magenta.
pub fn quantum_label(msg: &str) -> String {
    format!("{}", msg.truecolor(225, 0, 180).bold())
}

/// Format an Aeonmi keyword / output value in neon yellow.
pub fn aeonmi_value(msg: &str) -> String {
    format!("{}", msg.truecolor(255, 240, 0).bold())
}

/// Format a glyph-derived state hash as a colored string.
/// P4-16: state-hash → GDF color mapping.
pub fn state_hash_color(content: &str) -> String {
    use crate::glyph::gdf::{GlyphColor, GlyphParams};
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let hash = hasher.finalize();
    // SHA-256 always produces exactly 32 bytes. The assertion guards against
    // any future change to the hash length.
    debug_assert_eq!(hash.len(), 32, "SHA-256 must produce 32 bytes");

    // Build a minimal 64-byte seed from the 32-byte SHA-256 hash.
    // upper half = raw bytes, lower half = simple diffusion.
    let mut seed = [0u8; 64];
    for (i, &b) in hash.iter().take(32).enumerate() {
        seed[i] = b;
        seed[i + 32] = b.wrapping_mul(7).wrapping_add(13);
    }

    let params = GlyphParams::from_seed(&seed);
    let (r, g, b) = params.color.to_srgb();
    format!("{}", content.truecolor(r, g, b).bold())
}
