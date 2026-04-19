//! Boot Ceremony — unseal MGK, derive UGST, render glyph, return session.

use crate::glyph::anomaly::AnomalyDetector;
use crate::glyph::gdf::GlyphParams;
use crate::glyph::mgk::MasterGlyphKey;
use crate::glyph::ugst::{
    current_window, derive_glyph_seed, derive_ugst, derive_vault_key,
};
use anyhow::Result;

/// Everything produced by a successful boot ceremony.
pub struct BootResult {
    pub mgk: MasterGlyphKey,
    pub ugst: [u8; 64],
    pub glyph_seed: [u8; 64],
    pub glyph: GlyphParams,
    pub vault_key: [u8; 32],
    pub window: u64,
    pub anomaly_detector: AnomalyDetector,
}

/// Boot an existing shard: unseal MGK → derive UGST → render glyph.
pub fn boot(passphrase: &str) -> Result<BootResult> {
    // 1. Unseal MGK
    print_stage("1/4", "Unsealing Master Glyph Key...");
    let mgk = MasterGlyphKey::load_sealed(passphrase)?;
    println!("      ✓ MGK unsealed");

    // 2. Derive UGST for current window
    let window = current_window();
    print_stage("2/4", "Deriving UGST...");
    let ugst = derive_ugst(&mgk, window);
    println!("      ✓ UGST derived  [window={}]", window);

    // 3. Derive glyph seed and render
    print_stage("3/4", "Rendering Glyph...");
    let glyph_seed = derive_glyph_seed(&mgk, window, b"boot");
    let glyph = GlyphParams::from_seed(&glyph_seed);
    println!("{}", glyph.render_terminal());

    // 4. Derive vault key
    print_stage("4/4", "Opening Vault...");
    let vault_key = derive_vault_key(&mgk);
    println!("      ✓ Vault key ready");

    let anomaly_detector = AnomalyDetector::new(10, 60);

    Ok(BootResult {
        mgk,
        ugst,
        glyph_seed,
        glyph,
        vault_key,
        window,
        anomaly_detector,
    })
}

/// Initialize a new shard: generate MGK → seal → emit genesis glyph → return session.
pub fn init_shard(passphrase: &str) -> Result<BootResult> {
    print_stage("GEN", "Generating Master Glyph Key (UGST #0)...");
    let mgk = MasterGlyphKey::generate();

    print_stage("SEAL", "Sealing MGK with passphrase...");
    let saved_path = mgk.save_sealed(passphrase)?;
    println!("      ✓ MGK sealed → {}", saved_path.display());

    // Genesis window
    let window = current_window();
    let ugst = derive_ugst(&mgk, window);

    print_stage("GLYPH", "Rendering Genesis Glyph...");
    let glyph_seed = derive_glyph_seed(&mgk, window, b"genesis");
    let glyph = GlyphParams::from_seed(&glyph_seed);
    println!("{}", glyph.render_terminal());
    println!("      ✓ {}", glyph.status_line());

    let vault_key = derive_vault_key(&mgk);
    let anomaly_detector = AnomalyDetector::new(10, 60);

    println!("\n  ◈ SHARD INITIALIZED. Keep your passphrase safe. ◈\n");

    Ok(BootResult {
        mgk,
        ugst,
        glyph_seed,
        glyph,
        vault_key,
        window,
        anomaly_detector,
    })
}

fn print_stage(stage: &str, msg: &str) {
    println!("\n  [{}] {}", stage, msg);
}
