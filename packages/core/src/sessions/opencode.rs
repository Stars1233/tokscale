//! OpenCode session parser
//!
//! Parses messages from:
//! - SQLite database (OpenCode 1.2+): ~/.local/share/opencode/opencode.db
//! - Legacy JSON files: ~/.local/share/opencode/storage/message/

use super::{normalize_agent_name, UnifiedMessage};
use crate::TokenBreakdown;
use rusqlite::Connection;
use serde::Deserialize;
use std::path::Path;

/// OpenCode message structure (from JSON files and SQLite data column)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OpenCodeMessage {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "sessionID", default)]
    pub session_id: Option<String>,
    pub role: String,
    #[serde(rename = "modelID")]
    pub model_id: Option<String>,
    #[serde(rename = "providerID")]
    pub provider_id: Option<String>,
    pub cost: Option<f64>,
    pub tokens: Option<OpenCodeTokens>,
    pub time: OpenCodeTime,
    pub agent: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OpenCodeTokens {
    pub input: i64,
    pub output: i64,
    pub reasoning: Option<i64>,
    pub cache: OpenCodeCache,
}

#[derive(Debug, Deserialize)]
pub struct OpenCodeCache {
    pub read: i64,
    pub write: i64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OpenCodeTime {
    pub created: f64, // Unix timestamp in milliseconds (as float)
    pub completed: Option<f64>,
}

pub fn parse_opencode_file(path: &Path) -> Option<UnifiedMessage> {
    let data = std::fs::read(path).ok()?;
    let mut bytes = data;

    let msg: OpenCodeMessage = simd_json::from_slice(&mut bytes).ok()?;

    if msg.role != "assistant" {
        return None;
    }

    let tokens = msg.tokens?;
    let model_id = msg.model_id?;
    let agent_or_mode = msg.mode.or(msg.agent);
    let agent = agent_or_mode.map(|a| normalize_agent_name(&a));

    let session_id = msg.session_id.unwrap_or_else(|| "unknown".to_string());

    // Use message ID from JSON or derive from filename for deduplication
    let dedup_key = msg.id.or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    });

    let mut unified = UnifiedMessage::new_with_agent(
        "opencode",
        model_id,
        msg.provider_id.unwrap_or_else(|| "unknown".to_string()),
        session_id,
        msg.time.created as i64,
        TokenBreakdown {
            input: tokens.input.max(0),
            output: tokens.output.max(0),
            cache_read: tokens.cache.read.max(0),
            cache_write: tokens.cache.write.max(0),
            reasoning: tokens.reasoning.unwrap_or(0).max(0),
        },
        msg.cost.unwrap_or(0.0).max(0.0),
        agent,
    );
    unified.dedup_key = dedup_key;
    Some(unified)
}

pub fn parse_opencode_sqlite(db_path: &Path) -> Vec<UnifiedMessage> {
    let conn = match Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let query = r#"
        SELECT m.id, m.session_id, m.data
        FROM message m
        WHERE json_extract(m.data, '$.role') = 'assistant'
          AND json_extract(m.data, '$.tokens') IS NOT NULL
    "#;

    let mut stmt = match conn.prepare(query) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = match stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let session_id: String = row.get(1)?;
        let data_json: String = row.get(2)?;
        Ok((id, session_id, data_json))
    }) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut messages = Vec::new();

    for row_result in rows {
        let (id, session_id, data_json) = match row_result {
            Ok(r) => r,
            Err(_) => continue,
        };

        let mut bytes = data_json.into_bytes();
        let msg: OpenCodeMessage = match simd_json::from_slice(&mut bytes) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if msg.role != "assistant" {
            continue;
        }

        let tokens = match msg.tokens {
            Some(t) => t,
            None => continue,
        };

        let model_id = match msg.model_id {
            Some(m) => m,
            None => continue,
        };

        let agent_or_mode = msg.mode.or(msg.agent);
        let agent = agent_or_mode.map(|a| normalize_agent_name(&a));

        let mut unified = UnifiedMessage::new_with_agent(
            "opencode",
            model_id,
            msg.provider_id.unwrap_or_else(|| "unknown".to_string()),
            session_id,
            msg.time.created as i64,
            TokenBreakdown {
                input: tokens.input.max(0),
                output: tokens.output.max(0),
                cache_read: tokens.cache.read.max(0),
                cache_write: tokens.cache.write.max(0),
                reasoning: tokens.reasoning.unwrap_or(0).max(0),
            },
            msg.cost.unwrap_or(0.0).max(0.0),
            agent,
        );
        unified.dedup_key = Some(id);
        messages.push(unified);
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_opencode_structure() {
        let json = r#"{
            "id": "msg_123",
            "sessionID": "ses_456",
            "role": "assistant",
            "modelID": "claude-sonnet-4",
            "providerID": "anthropic",
            "cost": 0.05,
            "tokens": {
                "input": 1000,
                "output": 500,
                "reasoning": 100,
                "cache": { "read": 200, "write": 50 }
            },
            "time": { "created": 1700000000000.0 }
        }"#;

        let mut bytes = json.as_bytes().to_vec();
        let msg: OpenCodeMessage = simd_json::from_slice(&mut bytes).unwrap();

        assert_eq!(msg.model_id, Some("claude-sonnet-4".to_string()));
        assert_eq!(msg.tokens.unwrap().input, 1000);
        assert_eq!(msg.agent, None);
    }

    #[test]
    fn test_parse_opencode_with_agent() {
        let json = r#"{
            "id": "msg_123",
            "sessionID": "ses_456",
            "role": "assistant",
            "modelID": "claude-sonnet-4",
            "providerID": "anthropic",
            "agent": "OmO",
            "cost": 0.05,
            "tokens": {
                "input": 1000,
                "output": 500,
                "reasoning": 100,
                "cache": { "read": 200, "write": 50 }
            },
            "time": { "created": 1700000000000.0 }
        }"#;

        let mut bytes = json.as_bytes().to_vec();
        let msg: OpenCodeMessage = simd_json::from_slice(&mut bytes).unwrap();

        assert_eq!(msg.agent, Some("OmO".to_string()));
    }

    /// Verify negative token values are clamped to 0 (defense-in-depth for PR #147)
    #[test]
    fn test_negative_values_clamped_to_zero() {
        use std::io::Write;

        let json = r#"{
            "id": "msg_negative",
            "sessionID": "ses_negative",
            "role": "assistant",
            "modelID": "claude-sonnet-4",
            "providerID": "anthropic",
            "cost": -0.05,
            "tokens": {
                "input": -100,
                "output": -50,
                "reasoning": -25,
                "cache": { "read": -200, "write": -10 }
            },
            "time": { "created": 1700000000000.0 }
        }"#;

        let mut temp_file = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        temp_file.write_all(json.as_bytes()).unwrap();

        let result = parse_opencode_file(temp_file.path());
        assert!(result.is_some(), "Should parse file with negative values");

        let msg = result.unwrap();
        assert_eq!(msg.tokens.input, 0, "Negative input should be clamped to 0");
        assert_eq!(
            msg.tokens.output, 0,
            "Negative output should be clamped to 0"
        );
        assert_eq!(
            msg.tokens.cache_read, 0,
            "Negative cache_read should be clamped to 0"
        );
        assert_eq!(
            msg.tokens.cache_write, 0,
            "Negative cache_write should be clamped to 0"
        );
        assert_eq!(
            msg.tokens.reasoning, 0,
            "Negative reasoning should be clamped to 0"
        );
        assert!(
            msg.cost >= 0.0,
            "Negative cost should be clamped to 0.0, got {}",
            msg.cost
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    #[ignore] // Run manually with: cargo test integration -- --ignored
    fn test_parse_real_sqlite_db() {
        let home = std::env::var("HOME").unwrap();
        let db_path = PathBuf::from(format!("{}/.local/share/opencode/opencode.db", home));

        if !db_path.exists() {
            println!("Skipping: OpenCode database not found at {:?}", db_path);
            return;
        }

        let messages = parse_opencode_sqlite(&db_path);
        println!("Parsed {} messages from SQLite", messages.len());

        if !messages.is_empty() {
            let first = &messages[0];
            println!(
                "First message: model={}, provider={}, tokens={:?}",
                first.model_id, first.provider_id, first.tokens
            );
        }

        assert!(
            !messages.is_empty(),
            "Expected to parse some messages from SQLite"
        );
    }
}
