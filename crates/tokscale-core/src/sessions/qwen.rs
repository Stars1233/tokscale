//! Qwen CLI session parser
//!
//! Parses JSONL files from ~/.qwen/projects/{projectPath}/chats/*.jsonl
//! Token data comes from assistant messages with usageMetadata field.

use super::utils::{file_modified_timestamp_ms, parse_timestamp_str};
use super::UnifiedMessage;
use crate::TokenBreakdown;
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Qwen CLI JSONL line structure
#[derive(Debug, Deserialize)]
struct QwenLine {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    model: Option<String>,
    timestamp: Option<String>,
    #[serde(rename = "sessionId")]
    #[allow(dead_code)]
    session_id: Option<String>,

    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<i64>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<i64>,
    #[serde(rename = "thoughtsTokenCount")]
    thoughts_token_count: Option<i64>,
    #[serde(rename = "cachedContentTokenCount")]
    cached_content_token_count: Option<i64>,
}

/// Default model name when not specified
const DEFAULT_MODEL: &str = "unknown";
const DEFAULT_PROVIDER: &str = "qwen";

/// Extract session ID from the JSONL filename
/// Path format: ~/.qwen/projects/{project}/chats/{SESSION_ID}.jsonl
fn extract_session_id(path: &Path) -> String {
    path.file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Parse a Qwen CLI JSONL file
pub fn parse_qwen_file(path: &Path) -> Vec<UnifiedMessage> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let session_id = extract_session_id(path);
    let file_mtime = file_modified_timestamp_ms(path);

    let reader = BufReader::new(file);
    let mut messages: Vec<UnifiedMessage> = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut bytes = trimmed.as_bytes().to_vec();
        let qwen_line = match simd_json::from_slice::<QwenLine>(&mut bytes) {
            Ok(q) => q,
            Err(_) => continue,
        };

        // Only process assistant type messages with usageMetadata
        if qwen_line.msg_type.as_deref() != Some("assistant") {
            continue;
        }

        let usage = match qwen_line.usage_metadata {
            Some(u) => u,
            None => continue,
        };

        // Parse timestamp, fallback to file mtime
        let timestamp_ms = qwen_line
            .timestamp
            .and_then(|ts| parse_timestamp_str(&ts))
            .unwrap_or(file_mtime);

        // Extract token counts with defaults
        let input = usage.prompt_token_count.unwrap_or(0).max(0);
        let output = usage.candidates_token_count.unwrap_or(0).max(0);
        let reasoning = usage.thoughts_token_count.unwrap_or(0).max(0);
        let cache_read = usage.cached_content_token_count.unwrap_or(0).max(0);
        let cache_write = 0; // Qwen CLI doesn't report cache write tokens

        // Skip entries with zero tokens
        if input + output + cache_read + reasoning == 0 {
            continue;
        }

        // Use model from line or fallback to "unknown"
        let model = qwen_line.model.unwrap_or_else(|| DEFAULT_MODEL.to_string());

        messages.push(UnifiedMessage::new(
            "qwen",
            model,
            DEFAULT_PROVIDER,
            session_id.clone(),
            timestamp_ms,
            TokenBreakdown {
                input,
                output,
                cache_read,
                cache_write,
                reasoning,
            },
            0.0, // Cost calculated later by pricing resolver
        ));
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_parse_qwen_valid_assistant_message() {
        let content = r#"{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:24:56.857Z", "sessionId": "d96bf338", "usageMetadata": {"promptTokenCount": 12414, "candidatesTokenCount": 76, "thoughtsTokenCount": 39, "cachedContentTokenCount": 0}}"#;
        let file = create_test_file(content);

        let messages = parse_qwen_file(file.path());

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].client, "qwen");
        assert_eq!(messages[0].model_id, "qwen3.5-plus");
        assert_eq!(messages[0].provider_id, "qwen");
        // Session ID comes from filename, not JSON content (temp file has random name)
        assert!(!messages[0].session_id.is_empty());
        assert_eq!(messages[0].tokens.input, 12414);
        assert_eq!(messages[0].tokens.output, 76);
        assert_eq!(messages[0].tokens.reasoning, 39);
        assert_eq!(messages[0].tokens.cache_read, 0);
        assert_eq!(messages[0].tokens.cache_write, 0);
    }

    #[test]
    fn test_parse_qwen_multi_turn() {
        let content = r#"{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:24:56.857Z", "sessionId": "session1", "usageMetadata": {"promptTokenCount": 100, "candidatesTokenCount": 200, "thoughtsTokenCount": 10, "cachedContentTokenCount": 5}}
{"type": "assistant", "model": "qwen3-coder-plus", "timestamp": "2026-02-23T14:25:00.000Z", "sessionId": "session1", "usageMetadata": {"promptTokenCount": 300, "candidatesTokenCount": 400, "thoughtsTokenCount": 20, "cachedContentTokenCount": 10}}"#;
        let file = create_test_file(content);

        let messages = parse_qwen_file(file.path());

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].model_id, "qwen3.5-plus");
        assert_eq!(messages[0].tokens.input, 100);
        assert_eq!(messages[0].tokens.output, 200);
        assert_eq!(messages[0].tokens.reasoning, 10);
        assert_eq!(messages[0].tokens.cache_read, 5);
        assert_eq!(messages[1].model_id, "qwen3-coder-plus");
        assert_eq!(messages[1].tokens.input, 300);
        assert_eq!(messages[1].tokens.output, 400);
        assert_eq!(messages[1].tokens.reasoning, 20);
        assert_eq!(messages[1].tokens.cache_read, 10);
    }

    #[test]
    fn test_parse_qwen_skip_non_assistant() {
        let content = r#"{"type": "user", "timestamp": "2026-02-23T14:24:50.000Z", "content": "Hello"}
{"type": "system", "timestamp": "2026-02-23T14:24:51.000Z", "subtype": "ui_telemetry"}
{"type": "tool_result", "timestamp": "2026-02-23T14:24:52.000Z", "result": "success"}"#;
        let file = create_test_file(content);

        let messages = parse_qwen_file(file.path());

        assert!(messages.is_empty());
    }

    #[test]
    fn test_parse_qwen_empty_file() {
        let file = create_test_file("");

        let messages = parse_qwen_file(file.path());

        assert!(messages.is_empty());
    }

    #[test]
    fn test_parse_qwen_malformed_lines() {
        let content = r#"{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:24:56.857Z", "sessionId": "session1", "usageMetadata": {"promptTokenCount": 100, "candidatesTokenCount": 200, "thoughtsTokenCount": 10, "cachedContentTokenCount": 5}}
not valid json at all
{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:25:00.000Z", "sessionId": "session1", "usageMetadata": {"promptTokenCount": 300, "candidatesTokenCount": 400, "thoughtsTokenCount": 20, "cachedContentTokenCount": 10}}"#;
        let file = create_test_file(content);

        let messages = parse_qwen_file(file.path());

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].tokens.input, 100);
        assert_eq!(messages[1].tokens.input, 300);
    }

    #[test]
    fn test_parse_qwen_skips_zero_token_entries() {
        let content = r#"{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:24:56.857Z", "sessionId": "session1", "usageMetadata": {"promptTokenCount": 0, "candidatesTokenCount": 0, "thoughtsTokenCount": 0, "cachedContentTokenCount": 0}}"#;
        let file = create_test_file(content);

        let messages = parse_qwen_file(file.path());

        assert!(messages.is_empty());
    }

    #[test]
    fn test_parse_qwen_with_cache_and_reasoning() {
        let content = r#"{"type": "assistant", "model": "qwen3-max-2026-01-23", "timestamp": "2026-02-23T14:24:56.857Z", "sessionId": "session1", "usageMetadata": {"promptTokenCount": 1508, "candidatesTokenCount": 205, "thoughtsTokenCount": 50, "cachedContentTokenCount": 4864}}"#;
        let file = create_test_file(content);

        let messages = parse_qwen_file(file.path());

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].tokens.input, 1508);
        assert_eq!(messages[0].tokens.output, 205);
        assert_eq!(messages[0].tokens.reasoning, 50);
        assert_eq!(messages[0].tokens.cache_read, 4864);
        assert_eq!(messages[0].tokens.cache_write, 0);
    }

    #[test]
    fn test_parse_qwen_unknown_model_fallback() {
        let content = r#"{"type": "assistant", "timestamp": "2026-02-23T14:24:56.857Z", "sessionId": "session1", "usageMetadata": {"promptTokenCount": 100, "candidatesTokenCount": 200, "thoughtsTokenCount": 10, "cachedContentTokenCount": 5}}"#;
        let file = create_test_file(content);

        let messages = parse_qwen_file(file.path());

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].model_id, "unknown");
        assert_eq!(messages[0].tokens.input, 100);
    }
}
