use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use rayon::prelude::*;
use walkdir::WalkDir;

#[derive(Debug, Clone, Default)]
pub struct TokenBreakdown {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

impl TokenBreakdown {
    pub fn total(&self) -> u64 {
        self.input + self.output + self.cache_read + self.cache_write
    }
}

#[derive(Debug, Clone)]
pub struct ModelUsage {
    pub model: String,
    pub provider: String,
    pub source: String,
    pub tokens: TokenBreakdown,
    pub cost: f64,
    pub session_count: u32,
}

#[derive(Debug, Clone)]
pub struct DailyModelInfo {
    pub source: String,
    pub tokens: TokenBreakdown,
    pub cost: f64,
}

#[derive(Debug, Clone)]
pub struct DailyUsage {
    pub date: NaiveDate,
    pub tokens: TokenBreakdown,
    pub cost: f64,
    pub models: HashMap<String, DailyModelInfo>,
}

#[derive(Debug, Clone)]
pub struct ContributionDay {
    pub date: NaiveDate,
    pub tokens: u64,
    pub cost: f64,
    pub intensity: f64,
}

#[derive(Debug, Clone)]
pub struct GraphData {
    pub weeks: Vec<Vec<Option<ContributionDay>>>,
}

#[derive(Debug, Clone, Default)]
pub struct UsageData {
    pub models: Vec<ModelUsage>,
    pub daily: Vec<DailyUsage>,
    pub graph: Option<GraphData>,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub loading: bool,
    pub error: Option<String>,
    pub current_streak: u32,
    pub longest_streak: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    OpenCode,
    Claude,
    Codex,
    Cursor,
    Gemini,
    Amp,
    Droid,
    OpenClaw,
}

impl Source {
    pub fn all() -> &'static [Source] {
        &[
            Source::OpenCode,
            Source::Claude,
            Source::Codex,
            Source::Cursor,
            Source::Gemini,
            Source::Amp,
            Source::Droid,
            Source::OpenClaw,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Source::OpenCode => "OpenCode",
            Source::Claude => "Claude",
            Source::Codex => "Codex",
            Source::Cursor => "Cursor",
            Source::Gemini => "Gemini",
            Source::Amp => "Amp",
            Source::Droid => "Droid",
            Source::OpenClaw => "OpenClaw",
        }
    }

    pub fn key(&self) -> char {
        match self {
            Source::OpenCode => '1',
            Source::Claude => '2',
            Source::Codex => '3',
            Source::Cursor => '4',
            Source::Gemini => '5',
            Source::Amp => '6',
            Source::Droid => '7',
            Source::OpenClaw => '8',
        }
    }

    pub fn from_key(key: char) -> Option<Source> {
        match key {
            '1' => Some(Source::OpenCode),
            '2' => Some(Source::Claude),
            '3' => Some(Source::Codex),
            '4' => Some(Source::Cursor),
            '5' => Some(Source::Gemini),
            '6' => Some(Source::Amp),
            '7' => Some(Source::Droid),
            '8' => Some(Source::OpenClaw),
            _ => None,
        }
    }
}

struct ParsedMessage {
    source: String,
    model_id: String,
    provider_id: String,
    timestamp: i64,
    input: i64,
    output: i64,
    cache_read: i64,
    cache_write: i64,
    cost: f64,
}

pub struct DataLoader {
    _sessions_path: Option<PathBuf>,
}

impl DataLoader {
    pub fn new(sessions_path: Option<PathBuf>) -> Self {
        Self {
            _sessions_path: sessions_path,
        }
    }

    pub fn sessions_path(&self) -> Option<&PathBuf> {
        self._sessions_path.as_ref()
    }

    pub fn load(&self, enabled_sources: &[Source]) -> Result<UsageData> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .to_string_lossy()
            .to_string();

        let xdg_data =
            std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{}/.local/share", home));

        let mut all_messages: Vec<ParsedMessage> = Vec::new();

        for source in enabled_sources {
            match source {
                Source::OpenCode => {
                    let path = format!("{}/opencode/storage/message", xdg_data);
                    let files = scan_directory(&path, "*.json");
                    let msgs: Vec<ParsedMessage> =
                        files.par_iter().filter_map(parse_opencode_file).collect();
                    all_messages.extend(msgs);
                }
                Source::Claude => {
                    let path = format!("{}/.claude/projects", home);
                    let files = scan_directory(&path, "*.jsonl");
                    let msgs: Vec<ParsedMessage> =
                        files.par_iter().flat_map(parse_claude_file).collect();
                    all_messages.extend(msgs);
                }
                Source::Codex => {
                    let codex_home =
                        std::env::var("CODEX_HOME").unwrap_or_else(|_| format!("{}/.codex", home));
                    let path = format!("{}/sessions", codex_home);
                    let files = scan_directory(&path, "*.jsonl");
                    let msgs: Vec<ParsedMessage> =
                        files.par_iter().flat_map(parse_codex_file).collect();
                    all_messages.extend(msgs);
                }
                Source::Cursor => {
                    let path = format!("{}/.config/tokscale/cursor-cache", home);
                    let files = scan_cursor_files(&path);
                    let msgs: Vec<ParsedMessage> =
                        files.par_iter().flat_map(parse_cursor_file).collect();
                    all_messages.extend(msgs);
                }
                Source::Gemini => {
                    let path = format!("{}/.gemini/tmp", home);
                    let files = scan_directory(&path, "session-*.json");
                    let msgs: Vec<ParsedMessage> =
                        files.par_iter().flat_map(parse_gemini_file).collect();
                    all_messages.extend(msgs);
                }
                Source::Amp => {
                    let path = format!("{}/amp/threads", xdg_data);
                    let files = scan_directory(&path, "T-*.json");
                    let msgs: Vec<ParsedMessage> =
                        files.par_iter().flat_map(parse_amp_file).collect();
                    all_messages.extend(msgs);
                }
                Source::Droid => {
                    let path = format!("{}/.factory/sessions", home);
                    let files = scan_directory(&path, "*.settings.json");
                    let msgs: Vec<ParsedMessage> =
                        files.par_iter().flat_map(parse_droid_file).collect();
                    all_messages.extend(msgs);
                }
                Source::OpenClaw => {
                    for base in &[".openclaw", ".clawdbot", ".moltbot", ".moldbot"] {
                        let path = format!("{}/{}/agents", home, base);
                        let files = scan_directory(&path, "sessions.json");
                        let msgs: Vec<ParsedMessage> =
                            files.par_iter().flat_map(parse_openclaw_index).collect();
                        all_messages.extend(msgs);
                    }
                }
            }
        }

        let mut models_map: HashMap<String, ModelUsage> = HashMap::new();
        let mut daily_map: HashMap<NaiveDate, DailyUsage> = HashMap::new();
        let mut contribution_map: HashMap<String, (u64, f64)> = HashMap::new();

        for msg in all_messages {
            let key = format!("{}:{}:{}", msg.source, msg.provider_id, msg.model_id);

            let input = msg.input.max(0) as u64;
            let output = msg.output.max(0) as u64;
            let cache_read = msg.cache_read.max(0) as u64;
            let cache_write = msg.cache_write.max(0) as u64;
            let cost = if msg.cost.is_finite() && msg.cost >= 0.0 {
                msg.cost
            } else {
                0.0
            };

            let model_entry = models_map.entry(key).or_insert_with(|| ModelUsage {
                model: msg.model_id.clone(),
                provider: msg.provider_id.clone(),
                source: msg.source.clone(),
                tokens: TokenBreakdown::default(),
                cost: 0.0,
                session_count: 0,
            });
            model_entry.tokens.input += input;
            model_entry.tokens.output += output;
            model_entry.tokens.cache_read += cache_read;
            model_entry.tokens.cache_write += cache_write;
            model_entry.cost += cost;
            model_entry.session_count += 1;

            let date_str = timestamp_to_date(msg.timestamp);
            if let Ok(date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                let daily = daily_map.entry(date).or_insert_with(|| DailyUsage {
                    date,
                    tokens: TokenBreakdown::default(),
                    cost: 0.0,
                    models: HashMap::new(),
                });
                daily.tokens.input += input;
                daily.tokens.output += output;
                daily.tokens.cache_read += cache_read;
                daily.tokens.cache_write += cache_write;
                daily.cost += cost;

                let daily_model =
                    daily
                        .models
                        .entry(msg.model_id.clone())
                        .or_insert_with(|| DailyModelInfo {
                            source: msg.source.clone(),
                            tokens: TokenBreakdown::default(),
                            cost: 0.0,
                        });
                daily_model.tokens.input += input;
                daily_model.tokens.output += output;
                daily_model.tokens.cache_read += cache_read;
                daily_model.tokens.cache_write += cache_write;
                daily_model.cost += cost;

                let contribution = contribution_map.entry(date_str).or_insert((0, 0.0));
                contribution.0 += input + output + cache_read + cache_write;
                contribution.1 += cost;
            }
        }

        let models: Vec<ModelUsage> = models_map.into_values().collect();
        let total_tokens: u64 = models.iter().map(|m| m.tokens.total()).sum();
        let total_cost: f64 = models.iter().map(|m| m.cost).sum();

        let daily: Vec<DailyUsage> = daily_map.into_values().collect();
        let graph = build_graph_data(&contribution_map);

        // Calculate streaks
        let (current_streak, longest_streak) = calculate_streaks(&daily);

        Ok(UsageData {
            models,
            daily,
            graph: Some(graph),
            total_tokens,
            total_cost,
            loading: false,
            error: None,
            current_streak,
            longest_streak,
        })
    }
}

fn calculate_streaks(daily: &[DailyUsage]) -> (u32, u32) {
    if daily.is_empty() {
        return (0, 0);
    }

    // Sort dates
    let mut sorted_dates: Vec<NaiveDate> = daily.iter().map(|d| d.date).collect();
    sorted_dates.sort();

    let today = Utc::now().date_naive();

    // Calculate current streak (from most recent date going backwards)
    let mut current_streak = 0u32;
    if let Some(&most_recent) = sorted_dates.last() {
        let days_from_today = (today - most_recent).num_days();
        // Current streak only counts if most recent activity was today or yesterday
        if days_from_today <= 1 {
            current_streak = 1;
            for i in (0..sorted_dates.len() - 1).rev() {
                let diff = (sorted_dates[i + 1] - sorted_dates[i]).num_days();
                if diff == 1 {
                    current_streak += 1;
                } else {
                    break;
                }
            }
        }
    }

    // Calculate longest streak
    let mut longest_streak = 1u32;
    let mut streak = 1u32;
    for i in 1..sorted_dates.len() {
        let diff = (sorted_dates[i] - sorted_dates[i - 1]).num_days();
        if diff == 1 {
            streak += 1;
        } else {
            longest_streak = longest_streak.max(streak);
            streak = 1;
        }
    }
    longest_streak = longest_streak.max(streak);

    (current_streak, longest_streak)
}

fn timestamp_to_date(timestamp_ms: i64) -> String {
    match Utc.timestamp_millis_opt(timestamp_ms) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d").to_string(),
        _ => String::new(),
    }
}

fn scan_directory(root: &str, pattern: &str) -> Vec<PathBuf> {
    if !std::path::Path::new(root).exists() {
        return Vec::new();
    }

    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            if !path.is_file() {
                return false;
            }
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            match pattern {
                "*.json" => file_name.ends_with(".json"),
                "*.jsonl" => file_name.ends_with(".jsonl"),
                "session-*.json" => {
                    file_name.starts_with("session-") && file_name.ends_with(".json")
                }
                "T-*.json" => file_name.starts_with("T-") && file_name.ends_with(".json"),
                "*.settings.json" => file_name.ends_with(".settings.json"),
                "sessions.json" => file_name == "sessions.json",
                _ => false,
            }
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn scan_cursor_files(root: &str) -> Vec<PathBuf> {
    if !std::path::Path::new(root).exists() {
        return Vec::new();
    }

    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            if !path.is_file() {
                return false;
            }
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let is_archive = path.components().any(|c| {
                c.as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case("archive")
            });
            if is_archive {
                return false;
            }
            if file_name == "usage.csv" {
                return true;
            }
            if file_name.starts_with("usage.")
                && file_name.ends_with(".csv")
                && !file_name.starts_with("usage.backup")
            {
                return true;
            }
            false
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn detect_provider(model: &str) -> &'static str {
    let model_lower = model.to_lowercase();
    if model_lower.contains("claude")
        || model_lower.contains("sonnet")
        || model_lower.contains("opus")
        || model_lower.contains("haiku")
    {
        "Anthropic"
    } else if model_lower.contains("gpt")
        || model_lower.starts_with("o1")
        || model_lower.starts_with("o3")
        || model_lower.contains("codex")
    {
        "OpenAI"
    } else if model_lower.contains("gemini") {
        "Google"
    } else if model_lower.contains("deepseek") {
        "DeepSeek"
    } else if model_lower.contains("grok") {
        "xAI"
    } else if model_lower.contains("llama") || model_lower.contains("mixtral") {
        "Meta"
    } else if model_lower == "auto" || model_lower.contains("cursor") {
        "Cursor"
    } else {
        "Unknown"
    }
}

fn calculate_cost(model: &str, input: i64, output: i64, cache_read: i64, cache_write: i64) -> f64 {
    let model_lower = model.to_lowercase();
    let (input_cost, output_cost, cr_cost, cw_cost) = if model_lower.contains("opus") {
        (15.0, 75.0, 1.5, 18.75)
    } else if model_lower.contains("sonnet") {
        (3.0, 15.0, 0.3, 3.75)
    } else if model_lower.contains("haiku") {
        (0.25, 1.25, 0.025, 0.3125)
    } else if model_lower.contains("gpt-4o") {
        (2.5, 10.0, 1.25, 2.5)
    } else if model_lower.contains("gpt-4") {
        (10.0, 30.0, 5.0, 10.0)
    } else if model_lower.contains("gpt-3.5") {
        (0.5, 1.5, 0.25, 0.5)
    } else if model_lower.contains("gemini-2") || model_lower.contains("gemini-1.5-pro") {
        (1.25, 5.0, 0.3125, 1.25)
    } else if model_lower.contains("gemini-1.5-flash") {
        (0.075, 0.3, 0.01875, 0.075)
    } else if model_lower.contains("deepseek") {
        (0.14, 0.28, 0.014, 0.14)
    } else {
        (1.0, 3.0, 0.1, 1.0)
    };
    let per_million = 1_000_000.0;
    (input as f64 * input_cost / per_million)
        + (output as f64 * output_cost / per_million)
        + (cache_read as f64 * cr_cost / per_million)
        + (cache_write as f64 * cw_cost / per_million)
}

fn parse_opencode_file(path: &PathBuf) -> Option<ParsedMessage> {
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let model_id = json.get("modelID").and_then(|v| v.as_str())?.to_string();
    let provider_id = json
        .get("providerID")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| detect_provider(&model_id).to_string());

    let tokens = json.get("tokens")?;
    let input = tokens.get("input").and_then(|v| v.as_i64()).unwrap_or(0);
    let output = tokens.get("output").and_then(|v| v.as_i64()).unwrap_or(0);
    let cache = tokens.get("cache");
    let cache_read = cache
        .and_then(|c| c.get("read"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let cache_write = cache
        .and_then(|c| c.get("write"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    if input == 0 && output == 0 {
        return None;
    }

    let timestamp = json
        .get("time")
        .and_then(|t| t.get("created"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let cost = calculate_cost(&model_id, input, output, cache_read, cache_write);

    Some(ParsedMessage {
        source: "OpenCode".to_string(),
        model_id,
        provider_id,
        timestamp,
        input,
        output,
        cache_read,
        cache_write,
        cost,
    })
}

fn parse_claude_file(path: &PathBuf) -> Vec<ParsedMessage> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .filter_map(|line| {
            let json: serde_json::Value = serde_json::from_str(line).ok()?;

            let msg_type = json.get("type").and_then(|v| v.as_str())?;
            if msg_type != "assistant" {
                return None;
            }

            let message = json.get("message")?;
            let model_id = message.get("model").and_then(|v| v.as_str())?.to_string();
            let usage = message.get("usage")?;

            let input = usage
                .get("input_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let output = usage
                .get("output_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let cache_read = usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let cache_write = usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            if input == 0 && output == 0 {
                return None;
            }

            let timestamp = json
                .get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0);

            let provider_id = detect_provider(&model_id).to_string();
            let cost = calculate_cost(&model_id, input, output, cache_read, cache_write);

            Some(ParsedMessage {
                source: "Claude".to_string(),
                model_id,
                provider_id,
                timestamp,
                input,
                output,
                cache_read,
                cache_write,
                cost,
            })
        })
        .collect()
}

fn parse_codex_file(path: &PathBuf) -> Vec<ParsedMessage> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut current_model = "gpt-4o".to_string();
    let mut results = Vec::new();

    for line in content.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(msg_type) = json.get("type").and_then(|v| v.as_str()) {
                if msg_type == "model" {
                    if let Some(model) = json.get("model").and_then(|v| v.as_str()) {
                        current_model = model.to_string();
                    }
                } else if msg_type == "event_msg" {
                    if let Some(payload) = json.get("payload") {
                        if payload.get("type").and_then(|v| v.as_str()) == Some("token_count") {
                            if let Some(info) = payload.get("info") {
                                if let Some(usage) = info.get("last_token_usage") {
                                    let input = usage
                                        .get("input_tokens")
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(0);
                                    let output = usage
                                        .get("output_tokens")
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(0);

                                    if input > 0 || output > 0 {
                                        let timestamp = Utc::now().timestamp_millis();
                                        let provider_id =
                                            detect_provider(&current_model).to_string();
                                        let cost =
                                            calculate_cost(&current_model, input, output, 0, 0);

                                        results.push(ParsedMessage {
                                            source: "Codex".to_string(),
                                            model_id: current_model.clone(),
                                            provider_id,
                                            timestamp,
                                            input,
                                            output,
                                            cache_read: 0,
                                            cache_write: 0,
                                            cost,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    results
}

fn parse_gemini_file(path: &PathBuf) -> Vec<ParsedMessage> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let messages = match json.get("messages").and_then(|v| v.as_array()) {
        Some(m) => m,
        None => return Vec::new(),
    };

    messages
        .iter()
        .filter_map(|msg| {
            let msg_type = msg.get("type").and_then(|v| v.as_str())?;
            if msg_type != "gemini" {
                return None;
            }

            let model_id = msg.get("model").and_then(|v| v.as_str())?.to_string();
            let tokens = msg.get("tokens")?;

            let input = tokens.get("input").and_then(|v| v.as_i64()).unwrap_or(0);
            let output = tokens.get("output").and_then(|v| v.as_i64()).unwrap_or(0);
            let cache_read = tokens.get("cached").and_then(|v| v.as_i64()).unwrap_or(0);

            if input == 0 && output == 0 {
                return None;
            }

            let timestamp = msg.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
            let provider_id = "Google".to_string();
            let cost = calculate_cost(&model_id, input, output, cache_read, 0);

            Some(ParsedMessage {
                source: "Gemini".to_string(),
                model_id,
                provider_id,
                timestamp,
                input,
                output,
                cache_read,
                cache_write: 0,
                cost,
            })
        })
        .collect()
}

fn parse_cursor_file(path: &PathBuf) -> Vec<ParsedMessage> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .skip(1)
        .filter_map(|line| {
            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() < 9 {
                return None;
            }

            let timestamp = chrono::DateTime::parse_from_rfc3339(cols[0])
                .ok()
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0);
            let model_id = cols[1].to_string();
            let input: i64 = cols[3].parse().unwrap_or(0);
            let output: i64 = cols[4].parse().unwrap_or(0);
            let cache_read: i64 = cols[5].parse().unwrap_or(0);
            let cache_write: i64 = cols[6].parse().unwrap_or(0);
            let cost: f64 = cols[8].parse().unwrap_or(0.0);

            if input == 0 && output == 0 {
                return None;
            }

            let provider_id = detect_provider(&model_id).to_string();

            Some(ParsedMessage {
                source: "Cursor".to_string(),
                model_id,
                provider_id,
                timestamp,
                input,
                output,
                cache_read,
                cache_write,
                cost,
            })
        })
        .collect()
}

fn parse_amp_file(path: &PathBuf) -> Vec<ParsedMessage> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let messages = match json.get("messages").and_then(|v| v.as_array()) {
        Some(m) => m,
        None => return Vec::new(),
    };

    messages
        .iter()
        .filter_map(|msg| {
            let role = msg.get("role").and_then(|v| v.as_str())?;
            if role != "assistant" {
                return None;
            }

            let model_id = msg.get("model").and_then(|v| v.as_str())?.to_string();
            let usage = msg.get("usage")?;

            let input = usage
                .get("inputTokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let output = usage
                .get("outputTokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let cache_read = usage
                .get("cacheReadInputTokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let cache_write = usage
                .get("cacheCreationInputTokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            if input == 0 && output == 0 {
                return None;
            }

            let timestamp = msg.get("createdAt").and_then(|v| v.as_i64()).unwrap_or(0);
            let provider_id = detect_provider(&model_id).to_string();
            let cost = calculate_cost(&model_id, input, output, cache_read, cache_write);

            Some(ParsedMessage {
                source: "Amp".to_string(),
                model_id,
                provider_id,
                timestamp,
                input,
                output,
                cache_read,
                cache_write,
                cost,
            })
        })
        .collect()
}

fn parse_droid_file(path: &PathBuf) -> Vec<ParsedMessage> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let usage = match json.get("usage") {
        Some(u) => u,
        None => return Vec::new(),
    };

    let model_id = json
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let input = usage
        .get("inputTokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let output = usage
        .get("outputTokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let cache_read = usage
        .get("cacheReadInputTokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let cache_write = usage
        .get("cacheCreationInputTokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    if input == 0 && output == 0 {
        return Vec::new();
    }

    let timestamp = json
        .get("updatedAt")
        .and_then(|v| v.as_i64())
        .unwrap_or(Utc::now().timestamp_millis());
    let provider_id = detect_provider(&model_id).to_string();
    let cost = calculate_cost(&model_id, input, output, cache_read, cache_write);

    vec![ParsedMessage {
        source: "Droid".to_string(),
        model_id,
        provider_id,
        timestamp,
        input,
        output,
        cache_read,
        cache_write,
        cost,
    }]
}

fn parse_openclaw_index(path: &PathBuf) -> Vec<ParsedMessage> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let sessions = match json.as_object() {
        Some(s) => s,
        None => return Vec::new(),
    };

    let mut results = Vec::new();

    for (_key, session) in sessions {
        if let Some(session_file) = session.get("sessionFile").and_then(|v| v.as_str()) {
            let session_path = PathBuf::from(session_file);
            if let Ok(session_content) = std::fs::read_to_string(&session_path) {
                let mut current_model = "unknown".to_string();
                let mut current_provider = "unknown".to_string();

                for line in session_content.lines() {
                    if let Ok(msg_json) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(msg_type) = msg_json.get("type").and_then(|v| v.as_str()) {
                            if msg_type == "model_change" {
                                if let Some(model) =
                                    msg_json.get("modelId").and_then(|v| v.as_str())
                                {
                                    current_model = model.to_string();
                                    current_provider = detect_provider(&current_model).to_string();
                                }
                            } else if msg_type == "message" {
                                if let Some(message) = msg_json.get("message") {
                                    let role =
                                        message.get("role").and_then(|v| v.as_str()).unwrap_or("");
                                    if role != "assistant" {
                                        continue;
                                    }

                                    if let Some(usage) = message.get("usage") {
                                        let input = usage
                                            .get("input")
                                            .and_then(|v| v.as_i64())
                                            .unwrap_or(0);
                                        let output = usage
                                            .get("output")
                                            .and_then(|v| v.as_i64())
                                            .unwrap_or(0);
                                        let cache_read = usage
                                            .get("cacheRead")
                                            .and_then(|v| v.as_i64())
                                            .unwrap_or(0);

                                        if input == 0 && output == 0 {
                                            continue;
                                        }

                                        let timestamp = message
                                            .get("timestamp")
                                            .and_then(|v| v.as_i64())
                                            .unwrap_or(0);
                                        let cost = usage
                                            .get("cost")
                                            .and_then(|c| c.get("total"))
                                            .and_then(|v| v.as_f64())
                                            .unwrap_or_else(|| {
                                                calculate_cost(
                                                    &current_model,
                                                    input,
                                                    output,
                                                    cache_read,
                                                    0,
                                                )
                                            });

                                        results.push(ParsedMessage {
                                            source: "OpenClaw".to_string(),
                                            model_id: current_model.clone(),
                                            provider_id: current_provider.clone(),
                                            timestamp,
                                            input,
                                            output,
                                            cache_read,
                                            cache_write: 0,
                                            cost,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    results
}

fn build_graph_data(contributions: &HashMap<String, (u64, f64)>) -> GraphData {
    let today = Utc::now().date_naive();
    let mut weeks: Vec<Vec<Option<ContributionDay>>> = Vec::new();
    let mut current_week: Vec<Option<ContributionDay>> = Vec::new();

    let start_date = today - chrono::Duration::days(364);
    let start_weekday = start_date.weekday().num_days_from_sunday();

    for _ in 0..start_weekday {
        current_week.push(None);
    }

    let max_tokens = contributions.values().map(|(t, _)| *t).max().unwrap_or(1);

    for i in 0..=364 {
        let date = start_date + chrono::Duration::days(i);
        let date_str = date.format("%Y-%m-%d").to_string();

        let day = contributions
            .get(&date_str)
            .map(|(tokens, cost)| ContributionDay {
                date,
                tokens: *tokens,
                cost: *cost,
                intensity: if max_tokens > 0 {
                    *tokens as f64 / max_tokens as f64
                } else {
                    0.0
                },
            });

        current_week.push(day);

        if current_week.len() == 7 {
            weeks.push(current_week);
            current_week = Vec::new();
        }
    }

    if !current_week.is_empty() {
        while current_week.len() < 7 {
            current_week.push(None);
        }
        weeks.push(current_week);
    }

    GraphData { weeks }
}
