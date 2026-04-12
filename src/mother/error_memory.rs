//! ErrorMemory — Learn from mistakes, prevent repeated failures
//! Stores errors with context, queries for similar past errors, records fixes

use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)pub fn record_fix(&mut self, error_id: i64, fix: &str) -> SqlResult<()> {
        self.db.execute(
            "UPDATE errors SET fix = ?1 WHERE id = ?2",
            [fix, &error_id.to_string()pub fn get_top_errors(&self) -> SqlResult<Vec<PastError>> {
        let mut stmt = self.db.prepare(
            "SELECT id, timestamp, context, error_message, fix, occurrences 
             FROM errors 
             ORDER BY occurrences DESC
             LIMIT 10"
        )?;
        
        let errors = stmt.query_map([#[cfg(test)