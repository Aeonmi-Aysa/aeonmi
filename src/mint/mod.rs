//! Web3 Minting — NFT metadata + Anchor stub generation for Aeonmi artifacts.
//!
//! Re-exports the mint implementation from core. CLI: `aeonmi mint <file>`
pub use crate::core::mint::{Minter, MintMetadata, NftAttribute, AnchorStub};
