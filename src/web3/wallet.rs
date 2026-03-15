//! AEONMI Web3 Wallet
//!
//! Provides deterministic key-pair generation, address derivation, and an
//! in-memory balance ledger.  No real private keys ever leave the process; the
//! implementation is intentionally a **simulation layer** so that .ai programs
//! can experiment with Web3 concepts without touching a live network.
//!
//! # Quick start
//! ```
//! use aeonmi_project::web3::wallet::{Wallet, Ledger};
//!
//! let alice = Wallet::generate("alice");
//! let bob   = Wallet::generate("bob");
//!
//! let mut ledger = Ledger::new();
//! ledger.airdrop(&alice.address, 1000.0);
//! ledger.transfer(&alice.address, &bob.address, 250.0).unwrap();
//!
//! assert_eq!(ledger.balance(&alice.address), 750.0);
//! assert_eq!(ledger.balance(&bob.address),   250.0);
//! ```

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;

// ── Key pair ──────────────────────────────────────────────────────────────────

/// A deterministic, simulated Ed25519-style key pair.
#[derive(Debug, Clone)]
pub struct KeyPair {
    /// 32-byte private key (hex-encoded)
    pub private_key: String,
    /// 32-byte public key (hex-encoded)
    pub public_key: String,
}

impl KeyPair {
    /// Derive a key pair from a seed string (deterministic, for testing).
    pub fn from_seed(seed: &str) -> Self {
        let private_key = sha256_hex(&format!("priv:{}", seed));
        let public_key  = sha256_hex(&format!("pub:{}:{}", seed, &private_key[..16]));
        Self { private_key, public_key }
    }

    /// Sign a message — returns a deterministic hex "signature".
    pub fn sign(&self, message: &str) -> String {
        sha256_hex(&format!("sig:{}:{}", self.private_key, message))
    }

    /// Verify a signature produced by this key pair.
    pub fn verify(&self, message: &str, signature: &str) -> bool {
        self.sign(message) == signature
    }
}

// ── Address ───────────────────────────────────────────────────────────────────

/// Derives a human-readable wallet address from a public key.
///
/// Format: `AEON` + first 16 hex chars of SHA-256(public_key), e.g.
/// `AEON3f2a8b9c4d1e7f0a`.
pub fn derive_address(public_key: &str) -> String {
    let hash = sha256_hex(&format!("addr:{}", public_key));
    format!("AEON{}", &hash[..16])
}

// ── Wallet ────────────────────────────────────────────────────────────────────

/// A simulated Web3 wallet holding a key pair and its derived address.
#[derive(Debug, Clone)]
pub struct Wallet {
    pub name: String,
    pub keypair: KeyPair,
    pub address: String,
}

impl Wallet {
    /// Generate a new wallet from an arbitrary seed string.
    pub fn generate(seed: &str) -> Self {
        let keypair = KeyPair::from_seed(seed);
        let address = derive_address(&keypair.public_key);
        Self { name: seed.to_string(), keypair, address }
    }

    /// Sign a message with this wallet's private key.
    pub fn sign(&self, message: &str) -> String {
        self.keypair.sign(message)
    }

    /// Verify a message signature against this wallet's public key.
    pub fn verify(&self, message: &str, sig: &str) -> bool {
        self.keypair.verify(message, sig)
    }
}

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Wallet[{}] address={} pub={}...",
            self.name,
            self.address,
            &self.keypair.public_key[..12])
    }
}

// ── Ledger ────────────────────────────────────────────────────────────────────

/// Error type for ledger operations.
#[derive(Debug, Clone, PartialEq)]
pub enum LedgerError {
    InsufficientFunds { address: String, available: f64, requested: f64 },
    UnknownAddress(String),
    InvalidAmount(f64),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::InsufficientFunds { address, available, requested } =>
                write!(f, "InsufficientFunds: {} has {:.4} but {:.4} requested", address, available, requested),
            LedgerError::UnknownAddress(a) =>
                write!(f, "UnknownAddress: {}", a),
            LedgerError::InvalidAmount(a) =>
                write!(f, "InvalidAmount: {:.4}", a),
        }
    }
}

/// Represents a single transfer recorded in the ledger history.
#[derive(Debug, Clone)]
pub struct Transfer {
    pub from:   String,
    pub to:     String,
    pub amount: f64,
}

/// In-memory balance ledger for AEON token (or any simulated fungible token).
#[derive(Debug, Clone, Default)]
pub struct Ledger {
    balances: HashMap<String, f64>,
    /// Complete transfer history (append-only).
    pub history: Vec<Transfer>,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Credit `amount` AEON to `address` unconditionally (faucet / airdrop).
    pub fn airdrop(&mut self, address: &str, amount: f64) {
        *self.balances.entry(address.to_string()).or_insert(0.0) += amount;
        self.history.push(Transfer {
            from:   "SYSTEM".to_string(),
            to:     address.to_string(),
            amount,
        });
    }

    /// Return the balance of `address` (0 if never seen).
    pub fn balance(&self, address: &str) -> f64 {
        *self.balances.get(address).unwrap_or(&0.0)
    }

    /// Transfer `amount` from `from` to `to`.
    pub fn transfer(&mut self, from: &str, to: &str, amount: f64) -> Result<(), LedgerError> {
        if amount <= 0.0 {
            return Err(LedgerError::InvalidAmount(amount));
        }
        let from_bal = self.balance(from);
        if from_bal < amount {
            return Err(LedgerError::InsufficientFunds {
                address: from.to_string(),
                available: from_bal,
                requested: amount,
            });
        }
        *self.balances.entry(from.to_string()).or_insert(0.0) -= amount;
        *self.balances.entry(to.to_string()).or_insert(0.0) += amount;
        self.history.push(Transfer {
            from: from.to_string(),
            to: to.to_string(),
            amount,
        });
        Ok(())
    }

    /// Formatted summary table.
    pub fn summary(&self) -> String {
        let mut out = String::from("Address             | Balance\n");
        out.push_str("--------------------+-----------\n");
        let mut entries: Vec<_> = self.balances.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        for (addr, bal) in entries {
            out.push_str(&format!("{:<20}| {:.4}\n", addr, bal));
        }
        out
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn sha256_hex(data: &str) -> String {
    let mut h = Sha256::new();
    h.update(data.as_bytes());
    format!("{:x}", h.finalize())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_generate_deterministic() {
        let w1 = Wallet::generate("alice");
        let w2 = Wallet::generate("alice");
        assert_eq!(w1.address, w2.address);
        assert_eq!(w1.keypair.public_key, w2.keypair.public_key);
    }

    #[test]
    fn test_wallet_sign_verify() {
        let w = Wallet::generate("bob");
        let sig = w.sign("hello");
        assert!(w.verify("hello", &sig));
        assert!(!w.verify("tampered", &sig));
    }

    #[test]
    fn test_ledger_airdrop_and_balance() {
        let mut ledger = Ledger::new();
        let w = Wallet::generate("carol");
        ledger.airdrop(&w.address, 500.0);
        assert_eq!(ledger.balance(&w.address), 500.0);
    }

    #[test]
    fn test_ledger_transfer_ok() {
        let alice = Wallet::generate("alice");
        let bob   = Wallet::generate("bob");
        let mut ledger = Ledger::new();
        ledger.airdrop(&alice.address, 1000.0);
        ledger.transfer(&alice.address, &bob.address, 300.0).unwrap();
        assert_eq!(ledger.balance(&alice.address), 700.0);
        assert_eq!(ledger.balance(&bob.address),   300.0);
    }

    #[test]
    fn test_ledger_insufficient_funds() {
        let alice = Wallet::generate("alice");
        let bob   = Wallet::generate("bob");
        let mut ledger = Ledger::new();
        ledger.airdrop(&alice.address, 10.0);
        let err = ledger.transfer(&alice.address, &bob.address, 100.0).unwrap_err();
        assert!(matches!(err, LedgerError::InsufficientFunds { .. }));
    }

    #[test]
    fn test_ledger_invalid_amount() {
        let a = Wallet::generate("a");
        let b = Wallet::generate("b");
        let mut ledger = Ledger::new();
        assert!(matches!(
            ledger.transfer(&a.address, &b.address, -1.0),
            Err(LedgerError::InvalidAmount(_))
        ));
    }

    #[test]
    fn test_ledger_history() {
        let alice = Wallet::generate("alice");
        let bob   = Wallet::generate("bob");
        let mut ledger = Ledger::new();
        ledger.airdrop(&alice.address, 200.0);
        ledger.transfer(&alice.address, &bob.address, 50.0).unwrap();
        assert_eq!(ledger.history.len(), 2);
    }
}
