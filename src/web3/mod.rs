//! AEONMI Web3 subsystem.
//!
//! Three production-quality modules:
//!
//! | Module  | Purpose |
//! |---------|---------|
//! | [`wallet`] | Key-pair generation, address derivation, balance ledger |
//! | [`token`]  | ERC-20 / SPL fungible token (mint, burn, transfer, approve) |
//! | [`dao`]    | Governance: proposals, voting, tallying, execution |
//!
//! All modules are pure Rust with no external network calls; they are designed
//! to be embedded in `.ai` programs and exercised from the `aeonmi web3` CLI.

pub mod wallet;
pub mod token;
pub mod dao;
