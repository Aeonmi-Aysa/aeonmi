//! MGKS Bridge — Connects Aeonmi MGKS system to Mother's Rust core
//! Executes .ai scripts and syncs with genesis.json

use crate::runtime::Runtime;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)