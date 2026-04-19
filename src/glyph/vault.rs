//! Quantum Vault — XChaCha20-Poly1305 encrypted record store with Merkle integrity log.

use anyhow::{anyhow, Context, Result};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultRecord {
    pub id: String,
    pub record_type: String,
    pub label: String,
    /// XChaCha20-Poly1305 ciphertext (base64)
    pub ciphertext: String,
    /// Nonce (base64, 24 bytes)
    pub nonce: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlyphVault {
    pub records: Vec<VaultRecord>,
    pub merkle_root: String,
}

impl Default for GlyphVault {
    fn default() -> Self {
        Self::new()
    }
}

impl GlyphVault {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            merkle_root: "0".to_string(),
        }
    }

    /// Add an encrypted record.
    pub fn add_record(
        &mut self,
        vault_key: &[u8; 32],
        id: String,
        record_type: String,
        label: String,
        plaintext: &[u8],
    ) -> Result<()> {
        let cipher = XChaCha20Poly1305::new_from_slice(vault_key)
            .map_err(|e| anyhow!("key init: {e}"))?;
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow!("encrypt: {e}"))?;

        use base64::Engine;
        let record = VaultRecord {
            id,
            record_type,
            label,
            ciphertext: base64::engine::general_purpose::STANDARD.encode(&ciphertext),
            nonce: base64::engine::general_purpose::STANDARD.encode(nonce.as_slice()),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.records.push(record);
        self.update_merkle_root();
        Ok(())
    }

    /// Retrieve and decrypt a record by ID.
    pub fn get_record(&self, vault_key: &[u8; 32], id: &str) -> Result<Vec<u8>> {
        use base64::Engine;
        let record = self
            .records
            .iter()
            .find(|r| r.id == id)
            .ok_or_else(|| anyhow!("record '{}' not found", id))?;

        let ct = base64::engine::general_purpose::STANDARD
            .decode(&record.ciphertext)
            .context("decode ciphertext")?;
        let nonce_bytes = base64::engine::general_purpose::STANDARD
            .decode(&record.nonce)
            .context("decode nonce")?;
        let nonce = XNonce::from_slice(&nonce_bytes);

        let cipher = XChaCha20Poly1305::new_from_slice(vault_key)
            .map_err(|e| anyhow!("key init: {e}"))?;

        cipher.decrypt(nonce, ct.as_ref())
            .map_err(|_| anyhow!("decryption failed — wrong key or tampered record"))
    }

    /// Recompute the Merkle root from all record IDs and ciphertext hashes.
    pub fn update_merkle_root(&mut self) {
        self.merkle_root = self.compute_merkle_root();
    }

    /// Verify vault integrity by recomputing and comparing Merkle root.
    pub fn verify_integrity(&self) -> bool {
        self.compute_merkle_root() == self.merkle_root
    }

    pub fn merkle_root(&self) -> &str {
        &self.merkle_root
    }

    fn compute_merkle_root(&self) -> String {
        if self.records.is_empty() {
            return "0".to_string();
        }
        let mut leaves: Vec<[u8; 32]> = self
            .records
            .iter()
            .map(|r| {
                let mut hasher = Sha256::new();
                hasher.update(r.id.as_bytes());
                hasher.update(r.ciphertext.as_bytes());
                let hash: [u8; 32] = hasher.finalize().into();
                hash
            })
            .collect();

        while leaves.len() > 1 {
            let mut next = Vec::new();
            for chunk in leaves.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0]);
                if chunk.len() == 2 {
                    hasher.update(chunk[1]);
                } else {
                    hasher.update(chunk[0]); // odd leaf: duplicate
                }
                let h: [u8; 32] = hasher.finalize().into();
                next.push(h);
            }
            leaves = next;
        }
        hex::encode(leaves[0])
    }

    /// Default vault path: ~/.config/aeonmi/glyph_vault.json
    pub fn default_path() -> Result<PathBuf> {
        let mut path = dirs_next::home_dir()
            .ok_or_else(|| anyhow!("home dir not found"))?;
        path.push(".config");
        path.push("aeonmi");
        fs::create_dir_all(&path).context("create aeonmi config dir")?;
        path.push("glyph_vault.json");
        Ok(path)
    }

    /// Save vault to JSON.
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(self).context("serialize vault")?;
        fs::write(path, json).context("write vault")?;
        Ok(())
    }

    /// Load vault from JSON.
    pub fn load(path: &PathBuf) -> Result<Self> {
        let json = fs::read_to_string(path).context("read vault")?;
        let vault: Self = serde_json::from_str(&json).context("parse vault")?;
        Ok(vault)
    }

    /// Save to default location.
    pub fn save_default(&self) -> Result<PathBuf> {
        let path = Self::default_path()?;
        self.save(&path)?;
        Ok(path)
    }

    /// Load from default location; create fresh if not found.
    pub fn load_or_create() -> Result<Self> {
        let path = Self::default_path()?;
        if path.exists() {
            Self::load(&path)
        } else {
            Ok(Self::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [0xABu8; 32]
    }

    #[test]
    fn test_add_get_roundtrip() {
        let mut vault = GlyphVault::new();
        let key = test_key();
        vault
            .add_record(&key, "rec1".into(), "card".into(), "my card".into(), b"secret data")
            .unwrap();
        let plaintext = vault.get_record(&key, "rec1").unwrap();
        assert_eq!(plaintext, b"secret data");
    }

    #[test]
    fn test_wrong_key_fails() {
        let mut vault = GlyphVault::new();
        let key = test_key();
        vault
            .add_record(&key, "rec1".into(), "card".into(), "test".into(), b"data")
            .unwrap();
        let wrong_key = [0x00u8; 32];
        let result = vault.get_record(&wrong_key, "rec1");
        assert!(result.is_err(), "Wrong key should fail decryption");
    }

    #[test]
    fn test_merkle_integrity() {
        let mut vault = GlyphVault::new();
        let key = test_key();
        vault
            .add_record(&key, "r1".into(), "t".into(), "l".into(), b"hello")
            .unwrap();
        vault
            .add_record(&key, "r2".into(), "t".into(), "l2".into(), b"world")
            .unwrap();
        assert!(vault.verify_integrity(), "Integrity should pass before tampering");
        // Tamper
        vault.records[0].ciphertext = "TAMPERED".to_string();
        assert!(!vault.verify_integrity(), "Integrity should fail after tampering");
    }
}
