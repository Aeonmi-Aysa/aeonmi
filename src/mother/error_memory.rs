//! ErrorMemory — Learn from mistakes, prevent repeated failures
//! Stores errors with context, queries for similar past errors, records fixes

use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)pub fn log_error(&mut self, context: &str, error: &str) -> SqlResult<i64> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let existing: Option<i64> = self.db
            .query_row(
                "SELECT id FROM errors WHERE context = ?1 AND error_message = ?2",
                [context, errorpub fn record_fix(&mut self, error_id: i64, fix: &str) -> SqlResult<()> {
        self.db.execute(
            "UPDATE errors SET fix = ?1 WHERE id = ?2",
            [fix, &error_id.to_string()pub fn get_context_errors(&self, context: &str) -> SqlResult<Vec<PastError>> {
        let mut stmt = self.db.prepare(
            "SELECT id, timestamp, context, error_message, fix, occurrences 
             FROM errors 
             WHERE context = ?1
             ORDER BY timestamp DESC"
        )?;
        
        let errors = stmt.query_map([contextpub fn clear(&mut self) -> SqlResult<()> {
        self.db.execute("DELETE FROM errors", [#[cfg(test)#[test