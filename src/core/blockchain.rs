//! AEONMI Blockchain — SHA256 hash chain with quantum-signed artifacts
//! Ported from quantum_llama_bridge/src/blockchain/ledger.rs and extended.

use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ts() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

// ── Transaction ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    pub data: Option<String>, // arbitrary payload (e.g., glyph seed, NFT CID)
}

impl Transaction {
    pub fn new(sender: impl Into<String>, receiver: impl Into<String>, amount: f64) -> Self {
        Self { sender: sender.into(), receiver: receiver.into(), amount, data: None }
    }

    pub fn with_data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Serialise for hashing
    pub fn to_hash_input(&self) -> String {
        format!(
            "{}->{}:{}:{}",
            self.sender,
            self.receiver,
            self.amount,
            self.data.as_deref().unwrap_or("")
        )
    }
}

// ── Block ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

impl Block {
    pub fn genesis() -> Self {
        let mut b = Self {
            index: 0,
            timestamp: now_ts(),
            transactions: vec![],
            previous_hash: "0".to_string(),
            hash: String::new(),
            nonce: 0,
        };
        b.hash = b.calculate_hash();
        b
    }

    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let mut b = Self {
            index,
            timestamp: now_ts(),
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        };
        b.hash = b.calculate_hash();
        b
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string().as_bytes());
        hasher.update(self.timestamp.as_bytes());
        hasher.update(self.previous_hash.as_bytes());
        hasher.update(self.nonce.to_string().as_bytes());
        for tx in &self.transactions {
            hasher.update(tx.to_hash_input().as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Minimal proof-of-work: find nonce so hash starts with `difficulty` zeros
    pub fn mine(&mut self, difficulty: usize) {
        let prefix = "0".repeat(difficulty);
        while !self.hash.starts_with(&prefix) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }

    pub fn is_valid(&self) -> bool {
        self.hash == self.calculate_hash()
    }
}

// ── Blockchain ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending: Vec<Transaction>,
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            chain: vec![Block::genesis()],
            pending: Vec::new(),
        }
    }

    pub fn latest_hash(&self) -> &str {
        &self.chain.last().expect("chain always has genesis").hash
    }

    pub fn height(&self) -> u64 {
        self.chain.len() as u64
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending.push(tx);
    }

    /// Seal pending transactions into a new block
    pub fn seal_block(&mut self) -> &Block {
        let prev = self.latest_hash().to_string();
        let index = self.height();
        let txs = std::mem::take(&mut self.pending);
        let block = Block::new(index, txs, prev);
        self.chain.push(block);
        self.chain.last().unwrap()
    }

    /// Seal block with proof-of-work mining
    pub fn mine_block(&mut self, difficulty: usize) -> &Block {
        let prev = self.latest_hash().to_string();
        let index = self.height();
        let txs = std::mem::take(&mut self.pending);
        let mut block = Block::new(index, txs, prev);
        block.mine(difficulty);
        self.chain.push(block);
        self.chain.last().unwrap()
    }

    /// Verify the entire chain integrity
    pub fn verify(&self) -> bool {
        for i in 1..self.chain.len() {
            let curr = &self.chain[i];
            let prev = &self.chain[i - 1];
            if !curr.is_valid() {
                return false;
            }
            if curr.previous_hash != prev.hash {
                return false;
            }
        }
        true
    }

    pub fn summary(&self) -> String {
        format!(
            "Blockchain[height={} | latest={}...{}]",
            self.height(),
            &self.latest_hash()[..8],
            &self.latest_hash()[self.latest_hash().len() - 8..]
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_valid() {
        let chain = Blockchain::new();
        assert!(chain.chain[0].is_valid());
    }

    #[test]
    fn test_add_and_seal() {
        let mut chain = Blockchain::new();
        chain.add_transaction(Transaction::new("Alice", "Bob", 42.0));
        chain.seal_block();
        assert_eq!(chain.height(), 2);
        assert!(chain.verify());
    }

    #[test]
    fn test_tamper_detection() {
        let mut chain = Blockchain::new();
        chain.add_transaction(Transaction::new("Alice", "Bob", 10.0));
        chain.seal_block();
        // Tamper
        chain.chain[1].transactions[0].amount = 9999.0;
        assert!(!chain.verify());
    }

    #[test]
    fn test_multi_block() {
        let mut chain = Blockchain::new();
        for i in 0..5 {
            chain.add_transaction(Transaction::new(
                format!("sender{}", i),
                format!("recv{}", i),
                i as f64 * 1.5,
            ));
            chain.seal_block();
        }
        assert_eq!(chain.height(), 6);
        assert!(chain.verify());
    }
}
