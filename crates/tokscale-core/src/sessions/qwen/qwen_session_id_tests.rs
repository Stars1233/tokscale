//! TDD Tests for Qwen Session ID Resolution
//!
//! These tests verify that session IDs are correctly resolved to avoid collisions:
//! 1. Use JSON sessionId when present and non-empty
//! 2. Fallback to path-derived ID when sessionId is missing/empty
//! 3. Path-derived ID includes project path to prevent cross-project collisions

use super::{parse_qwen_file, extract_session_id_with_fallback};
use std::io::Write;
use tempfile::TempDir;
use std::path::Path;

fn create_test_file_with_name(content: &str, filename: &str) -> (TempDir, std::path::PathBuf) {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create a path that simulates the real Qwen structure
    // ~/.qwen/projects/{project}/chats/{filename}.jsonl
    let path = temp_dir.path().join(format!("test_project/chats/{}.jsonl", filename));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    (temp_dir, path)
}

/// Test 1: JSON has valid sessionId - should use JSON value
#[test]
fn test_session_id_from_json_when_present() {
    let content = r#"{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:24:56.857Z", "sessionId": "abc123def456", "usageMetadata": {"promptTokenCount": 100, "candidatesTokenCount": 200, "thoughtsTokenCount": 10, "cachedContentTokenCount": 5}}"#;
    let (_dir, path) = create_test_file_with_name(content, "json_present");
    
    let messages = parse_qwen_file(&path);
    
    assert_eq!(messages.len(), 1);
    // Should use the sessionId from JSON, not the filename
    assert_eq!(messages[0].session_id, "abc123def456");
}

/// Test 2: JSON sessionId is empty string - should fallback to path-derived ID
#[test]
fn test_session_id_fallback_when_empty_string() {
    let content = r#"{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:24:56.857Z", "sessionId": "", "usageMetadata": {"promptTokenCount": 100, "candidatesTokenCount": 200, "thoughtsTokenCount": 10, "cachedContentTokenCount": 5}}"#;
    let (_dir, path) = create_test_file_with_name(content, "json_empty");
    
    let messages = parse_qwen_file(&path);
    
    assert_eq!(messages.len(), 1);
    // Should fallback to path-derived ID (not empty string)
    assert!(!messages[0].session_id.is_empty());
    assert_ne!(messages[0].session_id, "");
    // Verify it's not the JSON empty value
    assert_ne!(messages[0].session_id, "");
}

/// Test 3: JSON has no sessionId field - should fallback to path-derived ID  
#[test]
fn test_session_id_fallback_when_missing() {
    let content = r#"{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:24:56.857Z", "usageMetadata": {"promptTokenCount": 100, "candidatesTokenCount": 200, "thoughtsTokenCount": 10, "cachedContentTokenCount": 5}}"#;
    let (_dir, path) = create_test_file_with_name(content, "json_missing");
    
    let messages = parse_qwen_file(&path);
    
    assert_eq!(messages.len(), 1);
    // Should fallback to path-derived ID
    assert!(!messages[0].session_id.is_empty());
}

/// Test 4: JSON sessionId is null - should fallback to path-derived ID
#[test]
fn test_session_id_fallback_when_null() {
    let content = r#"{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:24:56.857Z", "sessionId": null, "usageMetadata": {"promptTokenCount": 100, "candidatesTokenCount": 200, "thoughtsTokenCount": 10, "cachedContentTokenCount": 5}}"#;
    let (_dir, path) = create_test_file_with_name(content, "json_null");
    
    let messages = parse_qwen_file(&path);
    
    assert_eq!(messages.len(), 1);
    // Should fallback to path-derived ID
    assert!(!messages[0].session_id.is_empty());
    assert_ne!(messages[0].session_id, "null");
}

/// Test 5: Cross-project collision prevention - same filename in different projects
#[test]
fn test_cross_project_session_id_uniqueness() {
    let content = r#"{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:24:56.857Z", "usageMetadata": {"promptTokenCount": 100, "candidatesTokenCount": 200, "thoughtsTokenCount": 10, "cachedContentTokenCount": 5}}"#;
    
    // Create two files with same name in different projects
    let (_dir1, path1) = create_test_file_with_name(content, "session");
    
    // Manually create a second file in a different project
    let temp_dir = tempfile::tempdir().unwrap();
    let path2 = temp_dir.path().join("other_project/chats/session.jsonl");
    std::fs::create_dir_all(path2.parent().unwrap()).unwrap();
    let mut file2 = std::fs::File::create(&path2).unwrap();
    file2.write_all(content.as_bytes()).unwrap();
    
    let messages1 = parse_qwen_file(&path1);
    let messages2 = parse_qwen_file(&path2);
    
    assert_eq!(messages1.len(), 1);
    assert_eq!(messages2.len(), 1);
    
    // Session IDs should be different despite same filename
    assert_ne!(messages1[0].session_id, messages2[0].session_id);
}

/// Test 6: extract_session_id_with_fallback helper function - with valid sessionId
#[test]
fn test_extract_session_id_with_fallback_uses_json_value() {
    let path = Path::new("/home/user/.qwen/projects/myapp/chats/abc123.jsonl");
    let json_session_id = Some("json_session_456");
    
    let result = extract_session_id_with_fallback(path, json_session_id);
    
    assert_eq!(result, "json_session_456");
}

/// Test 7: extract_session_id_with_fallback helper function - with empty sessionId
#[test]
fn test_extract_session_id_with_fallback_empty_uses_path() {
    let path = Path::new("/home/user/.qwen/projects/myapp/chats/abc123.jsonl");
    let json_session_id = Some("");
    
    let result = extract_session_id_with_fallback(path, json_session_id);
    
    // Should use path-derived ID containing project and filename
    assert!(result.contains("myapp") || result.contains("abc123"));
}

/// Test 8: extract_session_id_with_fallback helper function - with None sessionId
#[test]
fn test_extract_session_id_with_fallback_none_uses_path() {
    let path = Path::new("/home/user/.qwen/projects/myapp/chats/abc123.jsonl");
    let json_session_id: Option<&str> = None;
    
    let result = extract_session_id_with_fallback(path, json_session_id);
    
    // Should use path-derived ID containing project and filename
    assert!(result.contains("myapp") || result.contains("abc123"));
}

/// Test 9: Path-derived session ID format includes project path
#[test]
fn test_path_derived_session_id_includes_project() {
    let path = Path::new("/home/user/.qwen/projects/some-project/chats/chat-session.jsonl");
    let result = extract_session_id_with_fallback(path, None);
    
    // Should include both project name and filename stem
    assert!(result.contains("some-project"), "Session ID should contain project name");
    assert!(result.contains("chat-session"), "Session ID should contain filename");
}

/// Test 10: Multi-turn messages within same file share the same session ID
#[test]
fn test_multi_turn_same_session_id() {
    let content = r#"{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:24:56.857Z", "sessionId": "shared_session", "usageMetadata": {"promptTokenCount": 100, "candidatesTokenCount": 200, "thoughtsTokenCount": 10, "cachedContentTokenCount": 5}}
{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:25:00.000Z", "sessionId": "shared_session", "usageMetadata": {"promptTokenCount": 300, "candidatesTokenCount": 400, "thoughtsTokenCount": 20, "cachedContentTokenCount": 10}}"#;
    let (_dir, path) = create_test_file_with_name(content, "multi");
    
    let messages = parse_qwen_file(&path);
    
    assert_eq!(messages.len(), 2);
    // Both messages should have the same session ID from JSON
    assert_eq!(messages[0].session_id, "shared_session");
    assert_eq!(messages[1].session_id, "shared_session");
}

/// Test 11: Mixed sessionId in same file (edge case) - uses fallback for empty/missing
#[test]
fn test_mixed_session_id_in_file() {
    let content = r#"{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:24:56.857Z", "sessionId": "valid_id", "usageMetadata": {"promptTokenCount": 100, "candidatesTokenCount": 200, "thoughtsTokenCount": 10, "cachedContentTokenCount": 5}}
{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:25:00.000Z", "usageMetadata": {"promptTokenCount": 300, "candidatesTokenCount": 400, "thoughtsTokenCount": 20, "cachedContentTokenCount": 10}}
{"type": "assistant", "model": "qwen3.5-plus", "timestamp": "2026-02-23T14:26:00.000Z", "sessionId": "", "usageMetadata": {"promptTokenCount": 500, "candidatesTokenCount": 600, "thoughtsTokenCount": 30, "cachedContentTokenCount": 15}}"#;
    let (_dir, path) = create_test_file_with_name(content, "mixed");
    
    let messages = parse_qwen_file(&path);
    
    assert_eq!(messages.len(), 3);
    // First message uses JSON sessionId
    assert_eq!(messages[0].session_id, "valid_id");
    // Second message (no sessionId) uses fallback
    assert!(messages[1].session_id.contains("mixed") || messages[1].session_id.contains("test_project"));
    // Third message (empty sessionId) uses fallback
    assert!(messages[2].session_id.contains("mixed") || messages[2].session_id.contains("test_project"));
}
