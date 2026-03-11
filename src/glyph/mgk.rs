//! Master Glyph Key — 256-bit root secret, sealed with Argon2id.

use anyhow::{anyhow, Context, Result};
use argon2::Argon2;
use argon2::password_hash::{rand_core::OsRng, SaltString};
use zeroize::Zeroize;
use std::fs;
use std::path::PathBuf;

/// 256-bit master secret. Zeroized on drop.
#[derive(Clone)]
pub struct MasterGlyphKey {
    bytes: [u8; 32],
}

impl Drop for MasterGlyphKey {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

impl std::fmt::Debug for MasterGlyphKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MasterGlyphKey([REDACTED])")
    }
}

impl MasterGlyphKey {
    /// Generate a fresh random 256-bit MGK.
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        Self { bytes }
    }

    /// Create from raw bytes (used during unsealing).
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Raw bytes — handle with care.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Seal the MGK with Argon2id + passphrase → hex-encoded ciphertext.
    pub fn seal(&self, passphrase: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        // Derive 32-byte key from passphrase using Argon2id
        let mut kek = [0u8; 32];
        argon2
            .hash_password_into(passphrase.as_bytes(), salt.as_str().as_bytes(), &mut kek)
            .map_err(|e| anyhow!("argon2 error: {e}"))?;

        // XOR-encrypt the MGK bytes with the derived key
        let mut ciphertext = [0u8; 32];
        for i in 0..32 {
            ciphertext[i] = self.bytes[i] ^ kek[i];
        }
        kek.zeroize();

        // Format: "AEONMIGLYPH:v1:<salt>:<hex_ciphertext>"
        let ct_hex = hex::encode(ciphertext);
        Ok(format!("AEONMIGLYPH:v1:{}:{}", salt.as_str(), ct_hex))
    }

    /// Unseal a previously sealed MGK blob.
    pub fn unseal(sealed: &str, passphrase: &str) -> Result<Self> {
        let parts: Vec<&str> = sealed.splitn(4, ':').collect();
        if parts.len() != 4 || parts[0] != "AEONMIGLYPH" || parts[1] != "v1" {
            return Err(anyhow!("Invalid sealed MGK format"));
        }
        let salt_str = parts[2];
        let ct_hex = parts[3];

        let argon2 = Argon2::default();
        let mut kek = [0u8; 32];
        argon2
            .hash_password_into(passphrase.as_bytes(), salt_str.as_bytes(), &mut kek)
            .map_err(|e| anyhow!("argon2 error: {e}"))?;

        let ciphertext = hex::decode(ct_hex).context("invalid hex in sealed MGK")?;
        if ciphertext.len() != 32 {
            kek.zeroize();
            return Err(anyhow!("Sealed MGK ciphertext wrong length"));
        }

        let mut mgk_bytes = [0u8; 32];
        for i in 0..32 {
            mgk_bytes[i] = ciphertext[i] ^ kek[i];
        }
        kek.zeroize();

        Ok(Self { bytes: mgk_bytes })
    }

    /// Default storage path: ~/.config/aeonmi/mgk.sealed
    pub fn storage_path() -> Result<PathBuf> {
        let mut path = dirs_next::home_dir().ok_or_else(|| anyhow!("home dir not found"))?;
        path.push(".config");
        path.push("aeonmi");
        fs::create_dir_all(&path).context("create aeonmi config dir")?;
        path.push("mgk.sealed");
        Ok(path)
    }

    /// Save the sealed blob to disk.
    pub fn save_sealed(&self, passphrase: &str) -> Result<PathBuf> {
        let sealed = self.seal(passphrase)?;
        let path = Self::storage_path()?;
        fs::write(&path, &sealed).context("write sealed MGK")?;
        Ok(path)
    }

    /// Load and unseal from disk.
    pub fn load_sealed(passphrase: &str) -> Result<Self> {
        let path = Self::storage_path()?;
        let sealed = fs::read_to_string(&path).context("read sealed MGK")?;
        Self::unseal(sealed.trim(), passphrase)
    }

    /// Returns true if a sealed MGK exists on disk.
    pub fn exists() -> bool {
        Self::storage_path().map(|p| p.exists()).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seal_unseal_roundtrip() {
        let mgk = MasterGlyphKey::generate();
        let original = *mgk.as_bytes();
        let sealed = mgk.seal("test_passphrase_42").unwrap();
        let recovered = MasterGlyphKey::unseal(&sealed, "test_passphrase_42").unwrap();
        assert_eq!(original, *recovered.as_bytes());
    }

    #[test]
    fn test_wrong_passphrase_gives_wrong_bytes() {
        let mgk = MasterGlyphKey::generate();
        let original = *mgk.as_bytes();
        let sealed = mgk.seal("correct_passphrase").unwrap();
        // Wrong passphrase should not panic but will produce wrong bytes
        let result = MasterGlyphKey::unseal(&sealed, "wrong_passphrase");
        // It may succeed (XOR cipher) but bytes differ
        if let Ok(wrong) = result {
            assert_ne!(original, *wrong.as_bytes(), "Wrong passphrase should not recover original bytes");
        }
    }

    #[test]
    fn test_format_prefix() {
        let mgk = MasterGlyphKey::generate();
        let sealed = mgk.seal("any").unwrap();
        assert!(sealed.starts_with("AEONMIGLYPH:v1:"), "Sealed should have proper format prefix");
    }
}
