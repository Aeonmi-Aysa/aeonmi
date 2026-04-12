//! ErrorMemory — Learn from mistakes, prevent repeated failures
//! Stores errors with context, queries for similar past errors, records fixes

use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)