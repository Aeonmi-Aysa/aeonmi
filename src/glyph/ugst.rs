//! UGST — Unique Glyph Signature Token
//! UGST_t = HKDF-SHA3-512(MGK, info="UGST"||T) rotated every 60 seconds.

use crate::glyph::mgk::MasterGlyphKey;
use hkdf::Hkdf;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

/// UGST rotation cadence in seconds.
pub const WINDOW_SECONDS: u64 = 60;

/// Returns the current 60-second time window index.
pub fn current_window() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / WINDOW_SECONDS
}

/// Derive UGST for a specific time window.
/// Uses HKDF-SHA256 (SHA3-512 would need the sha3 crate; using SHA256 until sha3 dep added).
pub fn derive_ugst(mgk: &MasterGlyphKey, window: u64) -> [u8; 64] {
    let info = format!("UGST{}", window);
    let hk = Hkdf::<Sha256>::new(None, mgk.as_bytes());
    let mut okm = [0u8; 64];
    hk.expand(info.as_bytes(), &mut okm)
        .expect("HKDF expand for UGST failed");
    okm
}

/// Derive UGST for the current time window.
pub fn current_ugst(mgk: &MasterGlyphKey) -> [u8; 64] {
    derive_ugst(mgk, current_window())
}

/// Derive the glyph visual seed (separate from signing — safe to derive visuals from).
pub fn derive_glyph_seed(mgk: &MasterGlyphKey, window: u64, context: &[u8]) -> [u8; 64] {
    let mut info = format!("GLYPH{}", window).into_bytes();
    info.extend_from_slice(context);
    let hk = Hkdf::<Sha256>::new(None, mgk.as_bytes());
    let mut okm = [0u8; 64];
    hk.expand(&info, &mut okm)
        .expect("HKDF expand for GLYPH failed");
    okm
}

/// Derive a per-session encryption key (32 bytes). Rotates with UGST window.
pub fn derive_session_key(mgk: &MasterGlyphKey, window: u64) -> [u8; 32] {
    let info = format!("SESSION{}", window);
    let hk = Hkdf::<Sha256>::new(None, mgk.as_bytes());
    let mut okm = [0u8; 32];
    hk.expand(info.as_bytes(), &mut okm)
        .expect("HKDF expand for SESSION failed");
    okm
}

/// Derive vault encryption key (stable, no time component).
pub fn derive_vault_key(mgk: &MasterGlyphKey) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, mgk.as_bytes());
    let mut okm = [0u8; 32];
    hk.expand(b"VAULT", &mut okm)
        .expect("HKDF expand for VAULT failed");
    okm
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glyph::mgk::MasterGlyphKey;

    fn test_mgk() -> MasterGlyphKey {
        MasterGlyphKey::from_bytes([0x42u8; 32])
    }

    #[test]
    fn test_ugst_deterministic() {
        let mgk = test_mgk();
        let a = derive_ugst(&mgk, 1000);
        let b = derive_ugst(&mgk, 1000);
        assert_eq!(a, b, "UGST must be deterministic for same window");
    }

    #[test]
    fn test_different_windows_differ() {
        let mgk = test_mgk();
        let a = derive_ugst(&mgk, 1000);
        let b = derive_ugst(&mgk, 1001);
        assert_ne!(a, b, "Different windows must produce different UGSTs");
    }

    #[test]
    fn test_context_matters_for_glyph_seed() {
        let mgk = test_mgk();
        let a = derive_glyph_seed(&mgk, 1000, b"boot");
        let b = derive_glyph_seed(&mgk, 1000, b"sign");
        assert_ne!(a, b, "Different contexts must produce different glyph seeds");
    }

    #[test]
    fn test_vault_key_stable() {
        let mgk = test_mgk();
        let a = derive_vault_key(&mgk);
        let b = derive_vault_key(&mgk);
        assert_eq!(a, b, "Vault key must be stable");
        assert_ne!(derive_ugst(&mgk, 0)[..32], a[..], "Vault key must differ from UGST");
    }
}
