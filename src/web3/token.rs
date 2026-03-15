//! AEONMI Web3 Token
//!
//! Implements an ERC-20 / SPL-inspired fungible token inside the AEONMI
//! runtime.  The token tracks total supply, per-account balances, and
//! spender allowances — all in memory.  It is designed to be embedded in
//! `.ai` programs or exercised from the `aeonmi token` CLI subcommand.
//!
//! # Example
//! ```
//! use aeonmi_project::web3::token::Token;
//!
//! let mut token = Token::new("Genesis Glyph Token", "GGT", 18, 1_000_000.0);
//! token.mint("AEON3f2a8b9c", 5000.0).unwrap();
//! token.transfer("AEON3f2a8b9c", "AEON0011aabb", 1000.0).unwrap();
//! assert_eq!(token.balance_of("AEON0011aabb"), 1000.0);
//! ```

use std::collections::HashMap;
use std::fmt;

// ── Error ─────────────────────────────────────────────────────────────────────

/// Errors returned by token operations.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenError {
    InsufficientBalance { account: String, have: f64, need: f64 },
    InsufficientAllowance { owner: String, spender: String, have: f64, need: f64 },
    OverflowSupply,
    InvalidAmount(f64),
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenError::InsufficientBalance { account, have, need } =>
                write!(f, "InsufficientBalance({}: have {:.4}, need {:.4})", account, have, need),
            TokenError::InsufficientAllowance { owner, spender, have, need } =>
                write!(f, "InsufficientAllowance(owner={}, spender={}: have {:.4}, need {:.4})", owner, spender, have, need),
            TokenError::OverflowSupply =>
                write!(f, "OverflowSupply: minting would exceed maximum supply"),
            TokenError::InvalidAmount(a) =>
                write!(f, "InvalidAmount({:.4})", a),
        }
    }
}

// ── Token ─────────────────────────────────────────────────────────────────────

/// A simulated ERC-20 / SPL fungible token.
#[derive(Debug, Clone)]
pub struct Token {
    /// Human-readable token name.
    pub name: String,
    /// Ticker symbol (e.g. "GGT").
    pub symbol: String,
    /// Number of decimal places (informational only — amounts are f64).
    pub decimals: u8,
    /// Maximum token supply (`f64::INFINITY` means uncapped).
    pub max_supply: f64,
    /// Current circulating supply.
    pub total_supply: f64,
    balances:   HashMap<String, f64>,
    allowances: HashMap<String, HashMap<String, f64>>, // owner -> spender -> amount
}

impl Token {
    /// Create a new token with an **initial supply of 0**.
    ///
    /// Use [`Token::new`] and then [`Token::mint`] to distribute tokens.
    pub fn new(name: &str, symbol: &str, decimals: u8, max_supply: f64) -> Self {
        Self {
            name:         name.to_string(),
            symbol:       symbol.to_string(),
            decimals,
            max_supply,
            total_supply: 0.0,
            balances:     HashMap::new(),
            allowances:   HashMap::new(),
        }
    }

    // ── Core ERC-20 operations ────────────────────────────────────────────────

    /// Return the balance of `account`.
    pub fn balance_of(&self, account: &str) -> f64 {
        *self.balances.get(account).unwrap_or(&0.0)
    }

    /// Mint `amount` new tokens into `account`.
    pub fn mint(&mut self, account: &str, amount: f64) -> Result<(), TokenError> {
        self.check_amount(amount)?;
        if self.total_supply + amount > self.max_supply {
            return Err(TokenError::OverflowSupply);
        }
        *self.balances.entry(account.to_string()).or_insert(0.0) += amount;
        self.total_supply += amount;
        Ok(())
    }

    /// Burn `amount` tokens from `account` (reduce supply).
    pub fn burn(&mut self, account: &str, amount: f64) -> Result<(), TokenError> {
        self.check_amount(amount)?;
        let bal = self.balance_of(account);
        if bal < amount {
            return Err(TokenError::InsufficientBalance {
                account: account.to_string(), have: bal, need: amount,
            });
        }
        *self.balances.entry(account.to_string()).or_insert(0.0) -= amount;
        self.total_supply -= amount;
        Ok(())
    }

    /// Transfer `amount` from `from` to `to`.
    pub fn transfer(&mut self, from: &str, to: &str, amount: f64) -> Result<(), TokenError> {
        self.check_amount(amount)?;
        let bal = self.balance_of(from);
        if bal < amount {
            return Err(TokenError::InsufficientBalance {
                account: from.to_string(), have: bal, need: amount,
            });
        }
        *self.balances.entry(from.to_string()).or_insert(0.0) -= amount;
        *self.balances.entry(to.to_string()).or_insert(0.0) += amount;
        Ok(())
    }

    // ── Allowance (approve / transferFrom) ───────────────────────────────────

    /// Approve `spender` to spend up to `amount` on behalf of `owner`.
    pub fn approve(&mut self, owner: &str, spender: &str, amount: f64) -> Result<(), TokenError> {
        self.check_amount(amount)?;
        self.allowances
            .entry(owner.to_string())
            .or_default()
            .insert(spender.to_string(), amount);
        Ok(())
    }

    /// Return the current allowance for `spender` granted by `owner`.
    pub fn allowance(&self, owner: &str, spender: &str) -> f64 {
        self.allowances
            .get(owner)
            .and_then(|m| m.get(spender))
            .copied()
            .unwrap_or(0.0)
    }

    /// Transfer `amount` from `from` to `to` using a pre-approved allowance.
    pub fn transfer_from(
        &mut self,
        from: &str,
        to: &str,
        spender: &str,
        amount: f64,
    ) -> Result<(), TokenError> {
        self.check_amount(amount)?;
        let allowed = self.allowance(from, spender);
        if allowed < amount {
            return Err(TokenError::InsufficientAllowance {
                owner: from.to_string(), spender: spender.to_string(),
                have: allowed, need: amount,
            });
        }
        self.transfer(from, to, amount)?;
        // Reduce allowance
        if let Some(m) = self.allowances.get_mut(from) {
            if let Some(a) = m.get_mut(spender) {
                *a -= amount;
            }
        }
        Ok(())
    }

    // ── Utility ───────────────────────────────────────────────────────────────

    /// Print a human-readable status summary.
    pub fn summary(&self) -> String {
        format!(
            "Token[{} ({})] decimals={} supply={:.4}/{:.4}",
            self.name,
            self.symbol,
            self.decimals,
            self.total_supply,
            if self.max_supply == f64::INFINITY { f64::MAX } else { self.max_supply }
        )
    }

    fn check_amount(&self, amount: f64) -> Result<(), TokenError> {
        if amount <= 0.0 || !amount.is_finite() {
            Err(TokenError::InvalidAmount(amount))
        } else {
            Ok(())
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ggt() -> Token {
        Token::new("Genesis Glyph Token", "GGT", 18, 1_000_000.0)
    }

    #[test]
    fn test_mint_and_balance() {
        let mut t = ggt();
        t.mint("alice", 1000.0).unwrap();
        assert_eq!(t.balance_of("alice"), 1000.0);
        assert_eq!(t.total_supply, 1000.0);
    }

    #[test]
    fn test_mint_exceeds_cap() {
        let mut t = Token::new("Capped", "CAP", 0, 100.0);
        assert!(matches!(t.mint("alice", 200.0), Err(TokenError::OverflowSupply)));
    }

    #[test]
    fn test_burn() {
        let mut t = ggt();
        t.mint("alice", 500.0).unwrap();
        t.burn("alice", 200.0).unwrap();
        assert_eq!(t.balance_of("alice"), 300.0);
        assert_eq!(t.total_supply, 300.0);
    }

    #[test]
    fn test_transfer_ok() {
        let mut t = ggt();
        t.mint("alice", 1000.0).unwrap();
        t.transfer("alice", "bob", 400.0).unwrap();
        assert_eq!(t.balance_of("alice"), 600.0);
        assert_eq!(t.balance_of("bob"),   400.0);
    }

    #[test]
    fn test_transfer_insufficient() {
        let mut t = ggt();
        t.mint("alice", 100.0).unwrap();
        assert!(matches!(
            t.transfer("alice", "bob", 200.0),
            Err(TokenError::InsufficientBalance { .. })
        ));
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let mut t = ggt();
        t.mint("alice", 1000.0).unwrap();
        t.approve("alice", "spender", 300.0).unwrap();
        t.transfer_from("alice", "carol", "spender", 200.0).unwrap();
        assert_eq!(t.balance_of("carol"),  200.0);
        assert_eq!(t.allowance("alice", "spender"), 100.0);
    }

    #[test]
    fn test_allowance_exceeded() {
        let mut t = ggt();
        t.mint("alice", 1000.0).unwrap();
        t.approve("alice", "spender", 50.0).unwrap();
        assert!(matches!(
            t.transfer_from("alice", "carol", "spender", 100.0),
            Err(TokenError::InsufficientAllowance { .. })
        ));
    }

    #[test]
    fn test_invalid_amount() {
        let mut t = ggt();
        assert!(matches!(t.mint("alice", 0.0),  Err(TokenError::InvalidAmount(_))));
        assert!(matches!(t.mint("alice", -1.0), Err(TokenError::InvalidAmount(_))));
    }
}
