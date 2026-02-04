#![deny(clippy::all)]

mod aggregator;
mod parser;
pub mod pricing;
pub mod scanner;
pub mod sessions;

pub use aggregator::*;
pub use parser::*;
pub use scanner::*;
pub use sessions::UnifiedMessage;

use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn health_check() -> String {
    "tokscale-core is healthy!".to_string()
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct TokenBreakdown {
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_write: i64,
    pub reasoning: i64,
}

impl TokenBreakdown {
    pub fn total(&self) -> i64 {
        self.input + self.output + self.cache_read + self.cache_write + self.reasoning
    }
}

#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub source: String,
    pub model_id: String,
    pub provider_id: String,
    pub session_id: String,
    pub timestamp: i64,
    pub date: String,
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_write: i64,
    pub reasoning: i64,
    pub agent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedMessages {
    pub messages: Vec<ParsedMessage>,
    pub opencode_count: i32,
    pub claude_count: i32,
    pub codex_count: i32,
    pub gemini_count: i32,
    pub amp_count: i32,
    pub droid_count: i32,
    pub openclaw_count: i32,
    pub processing_time_ms: u32,
}

#[derive(Debug, Clone)]
pub struct LocalParseOptions {
    pub home_dir: Option<String>,
    pub sources: Option<Vec<String>>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub year: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct DailyTotals {
    pub tokens: i64,
    pub cost: f64,
    pub messages: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceContribution {
    pub source: String,
    pub model_id: String,
    pub provider_id: String,
    pub tokens: TokenBreakdown,
    pub cost: f64,
    pub messages: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DailyContribution {
    pub date: String,
    pub totals: DailyTotals,
    pub intensity: u8,
    pub token_breakdown: TokenBreakdown,
    pub sources: Vec<SourceContribution>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct YearSummary {
    pub year: String,
    pub total_tokens: i64,
    pub total_cost: f64,
    pub range_start: String,
    pub range_end: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DataSummary {
    pub total_tokens: i64,
    pub total_cost: f64,
    pub total_days: i32,
    pub active_days: i32,
    pub average_per_day: f64,
    pub max_cost_in_single_day: f64,
    pub sources: Vec<String>,
    pub models: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphMeta {
    pub generated_at: String,
    pub version: String,
    pub date_range_start: String,
    pub date_range_end: String,
    pub processing_time_ms: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphResult {
    pub meta: GraphMeta,
    pub summary: DataSummary,
    pub years: Vec<YearSummary>,
    pub contributions: Vec<DailyContribution>,
}

#[derive(Debug, Clone)]
pub struct ReportOptions {
    pub home_dir: Option<String>,
    pub sources: Option<Vec<String>>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub year: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelUsage {
    pub source: String,
    pub model: String,
    pub provider: String,
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_write: i64,
    pub reasoning: i64,
    pub message_count: i32,
    pub cost: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MonthlyUsage {
    pub month: String,
    pub models: Vec<String>,
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_write: i64,
    pub message_count: i32,
    pub cost: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelReport {
    pub entries: Vec<ModelUsage>,
    pub total_input: i64,
    pub total_output: i64,
    pub total_cache_read: i64,
    pub total_cache_write: i64,
    pub total_messages: i32,
    pub total_cost: f64,
    pub processing_time_ms: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MonthlyReport {
    pub entries: Vec<MonthlyUsage>,
    pub total_cost: f64,
    pub processing_time_ms: u32,
}

pub fn get_home_dir_string(home_dir_option: &Option<String>) -> Result<String, String> {
    home_dir_option
        .clone()
        .or_else(|| std::env::var("HOME").ok())
        .or_else(|| dirs::home_dir().map(|p| p.to_string_lossy().into_owned()))
        .ok_or_else(|| {
            "HOME directory not specified and could not determine home directory".to_string()
        })
}

fn parse_all_messages_with_pricing(
    home_dir: &str,
    sources: &[String],
    pricing: &pricing::PricingService,
) -> Vec<UnifiedMessage> {
    let scan_result = scanner::scan_all_sources(home_dir, sources);
    let mut all_messages: Vec<UnifiedMessage> = Vec::new();

    let opencode_messages: Vec<UnifiedMessage> = scan_result
        .opencode_files
        .par_iter()
        .filter_map(|path| {
            let mut msg = sessions::opencode::parse_opencode_file(path)?;
            msg.cost = pricing.calculate_cost(
                &msg.model_id,
                msg.tokens.input,
                msg.tokens.output,
                msg.tokens.cache_read,
                msg.tokens.cache_write,
                msg.tokens.reasoning,
            );
            Some(msg)
        })
        .collect();
    all_messages.extend(opencode_messages);

    let claude_messages: Vec<UnifiedMessage> = scan_result
        .claude_files
        .par_iter()
        .flat_map(|path| {
            sessions::claudecode::parse_claude_file(path)
                .into_iter()
                .map(|mut msg| {
                    msg.cost = pricing.calculate_cost(
                        &msg.model_id,
                        msg.tokens.input,
                        msg.tokens.output,
                        msg.tokens.cache_read,
                        msg.tokens.cache_write,
                        msg.tokens.reasoning,
                    );
                    msg
                })
                .collect::<Vec<_>>()
        })
        .collect();
    all_messages.extend(claude_messages);

    let codex_messages: Vec<UnifiedMessage> = scan_result
        .codex_files
        .par_iter()
        .flat_map(|path| {
            sessions::codex::parse_codex_file(path)
                .into_iter()
                .map(|mut msg| {
                    msg.cost = pricing.calculate_cost(
                        &msg.model_id,
                        msg.tokens.input,
                        msg.tokens.output,
                        msg.tokens.cache_read,
                        msg.tokens.cache_write,
                        msg.tokens.reasoning,
                    );
                    msg
                })
                .collect::<Vec<_>>()
        })
        .collect();
    all_messages.extend(codex_messages);

    let gemini_messages: Vec<UnifiedMessage> = scan_result
        .gemini_files
        .par_iter()
        .flat_map(|path| {
            sessions::gemini::parse_gemini_file(path)
                .into_iter()
                .map(|mut msg| {
                    msg.cost = pricing.calculate_cost(
                        &msg.model_id,
                        msg.tokens.input,
                        msg.tokens.output + msg.tokens.reasoning,
                        0,
                        0,
                        0,
                    );
                    msg
                })
                .collect::<Vec<_>>()
        })
        .collect();
    all_messages.extend(gemini_messages);

    let cursor_messages: Vec<UnifiedMessage> = scan_result
        .cursor_files
        .par_iter()
        .flat_map(|path| {
            sessions::cursor::parse_cursor_file(path)
                .into_iter()
                .map(|mut msg| {
                    let csv_cost = msg.cost;
                    let calculated_cost = pricing.calculate_cost(
                        &msg.model_id,
                        msg.tokens.input,
                        msg.tokens.output,
                        msg.tokens.cache_read,
                        msg.tokens.cache_write,
                        msg.tokens.reasoning,
                    );
                    msg.cost = if calculated_cost > 0.0 {
                        calculated_cost
                    } else {
                        csv_cost
                    };
                    msg
                })
                .collect::<Vec<_>>()
        })
        .collect();
    all_messages.extend(cursor_messages);

    let amp_messages: Vec<UnifiedMessage> = scan_result
        .amp_files
        .par_iter()
        .flat_map(|path| {
            sessions::amp::parse_amp_file(path)
                .into_iter()
                .map(|mut msg| {
                    let credits = msg.cost;
                    let calculated_cost = pricing.calculate_cost(
                        &msg.model_id,
                        msg.tokens.input,
                        msg.tokens.output,
                        msg.tokens.cache_read,
                        msg.tokens.cache_write,
                        msg.tokens.reasoning,
                    );
                    msg.cost = if calculated_cost > 0.0 {
                        calculated_cost
                    } else {
                        credits
                    };
                    msg
                })
                .collect::<Vec<_>>()
        })
        .collect();
    all_messages.extend(amp_messages);

    let droid_messages: Vec<UnifiedMessage> = scan_result
        .droid_files
        .par_iter()
        .flat_map(|path| {
            sessions::droid::parse_droid_file(path)
                .into_iter()
                .map(|mut msg| {
                    msg.cost = pricing.calculate_cost(
                        &msg.model_id,
                        msg.tokens.input,
                        msg.tokens.output,
                        msg.tokens.cache_read,
                        msg.tokens.cache_write,
                        msg.tokens.reasoning,
                    );
                    msg
                })
                .collect::<Vec<_>>()
        })
        .collect();
    all_messages.extend(droid_messages);

    let openclaw_messages: Vec<UnifiedMessage> = scan_result
        .openclaw_files
        .par_iter()
        .flat_map(|path| {
            sessions::openclaw::parse_openclaw_index(path)
                .into_iter()
                .map(|mut msg| {
                    msg.cost = pricing.calculate_cost(
                        &msg.model_id,
                        msg.tokens.input,
                        msg.tokens.output,
                        msg.tokens.cache_read,
                        msg.tokens.cache_write,
                        msg.tokens.reasoning,
                    );
                    msg
                })
                .collect::<Vec<_>>()
        })
        .collect();
    all_messages.extend(openclaw_messages);

    all_messages
}

pub async fn get_model_report(options: ReportOptions) -> Result<ModelReport, String> {
    let start = Instant::now();

    let home_dir = get_home_dir_string(&options.home_dir)?;

    let sources = options.sources.clone().unwrap_or_else(|| {
        vec![
            "opencode".to_string(),
            "claude".to_string(),
            "codex".to_string(),
            "gemini".to_string(),
            "cursor".to_string(),
            "amp".to_string(),
            "droid".to_string(),
            "openclaw".to_string(),
        ]
    });

    let pricing = pricing::PricingService::get_or_init().await?;
    let all_messages = parse_all_messages_with_pricing(&home_dir, &sources, &pricing);

    let filtered = filter_messages_for_report(all_messages, &options);

    let mut model_map: HashMap<String, ModelUsage> = HashMap::new();

    for msg in filtered {
        let key = format!("{}:{}:{}", msg.source, msg.provider_id, msg.model_id);
        let entry = model_map.entry(key).or_insert_with(|| ModelUsage {
            source: msg.source.clone(),
            model: msg.model_id.clone(),
            provider: msg.provider_id.clone(),
            input: 0,
            output: 0,
            cache_read: 0,
            cache_write: 0,
            reasoning: 0,
            message_count: 0,
            cost: 0.0,
        });

        entry.input += msg.tokens.input;
        entry.output += msg.tokens.output;
        entry.cache_read += msg.tokens.cache_read;
        entry.cache_write += msg.tokens.cache_write;
        entry.reasoning += msg.tokens.reasoning;
        entry.message_count += 1;
        entry.cost += msg.cost;
    }

    let mut entries: Vec<ModelUsage> = model_map.into_values().collect();
    entries.sort_by(|a, b| {
        match (a.cost.is_nan(), b.cost.is_nan()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => b
                .cost
                .partial_cmp(&a.cost)
                .unwrap_or(std::cmp::Ordering::Equal),
        }
    });

    let total_input: i64 = entries.iter().map(|e| e.input).sum();
    let total_output: i64 = entries.iter().map(|e| e.output).sum();
    let total_cache_read: i64 = entries.iter().map(|e| e.cache_read).sum();
    let total_cache_write: i64 = entries.iter().map(|e| e.cache_write).sum();
    let total_messages: i32 = entries.iter().map(|e| e.message_count).sum();
    let total_cost: f64 = entries.iter().map(|e| e.cost).sum();

    Ok(ModelReport {
        entries,
        total_input,
        total_output,
        total_cache_read,
        total_cache_write,
        total_messages,
        total_cost,
        processing_time_ms: start.elapsed().as_millis() as u32,
    })
}

#[derive(Default)]
struct MonthAggregator {
    models: HashSet<String>,
    input: i64,
    output: i64,
    cache_read: i64,
    cache_write: i64,
    message_count: i32,
    cost: f64,
}

pub async fn get_monthly_report(options: ReportOptions) -> Result<MonthlyReport, String> {
    let start = Instant::now();

    let home_dir = get_home_dir_string(&options.home_dir)?;

    let sources = options.sources.clone().unwrap_or_else(|| {
        vec![
            "opencode".to_string(),
            "claude".to_string(),
            "codex".to_string(),
            "gemini".to_string(),
            "cursor".to_string(),
            "amp".to_string(),
            "droid".to_string(),
            "openclaw".to_string(),
        ]
    });

    let pricing = pricing::PricingService::get_or_init().await?;
    let all_messages = parse_all_messages_with_pricing(&home_dir, &sources, &pricing);

    let filtered = filter_messages_for_report(all_messages, &options);

    let mut month_map: HashMap<String, MonthAggregator> = HashMap::new();

    for msg in filtered {
        let month = if msg.date.len() >= 7 {
            msg.date[..7].to_string()
        } else {
            continue;
        };

        let entry = month_map.entry(month).or_default();

        entry.models.insert(msg.model_id.clone());
        entry.input += msg.tokens.input;
        entry.output += msg.tokens.output;
        entry.cache_read += msg.tokens.cache_read;
        entry.cache_write += msg.tokens.cache_write;
        entry.message_count += 1;
        entry.cost += msg.cost;
    }

    let mut entries: Vec<MonthlyUsage> = month_map
        .into_iter()
        .map(|(month, agg)| MonthlyUsage {
            month,
            models: agg.models.into_iter().collect(),
            input: agg.input,
            output: agg.output,
            cache_read: agg.cache_read,
            cache_write: agg.cache_write,
            message_count: agg.message_count,
            cost: agg.cost,
        })
        .collect();

    entries.sort_by(|a, b| a.month.cmp(&b.month));

    let total_cost: f64 = entries.iter().map(|e| e.cost).sum();

    Ok(MonthlyReport {
        entries,
        total_cost,
        processing_time_ms: start.elapsed().as_millis() as u32,
    })
}

pub async fn generate_graph(options: ReportOptions) -> Result<GraphResult, String> {
    let start = Instant::now();

    let home_dir = get_home_dir_string(&options.home_dir)?;

    let sources = options.sources.clone().unwrap_or_else(|| {
        vec![
            "opencode".to_string(),
            "claude".to_string(),
            "codex".to_string(),
            "gemini".to_string(),
            "cursor".to_string(),
            "amp".to_string(),
            "droid".to_string(),
            "openclaw".to_string(),
        ]
    });

    let pricing = pricing::PricingService::get_or_init().await?;
    let all_messages = parse_all_messages_with_pricing(&home_dir, &sources, &pricing);

    let filtered = filter_messages_for_report(all_messages, &options);

    let contributions = aggregator::aggregate_by_date(filtered);

    let processing_time_ms = start.elapsed().as_millis() as u32;
    let result = aggregator::generate_graph_result(contributions, processing_time_ms);

    Ok(result)
}

fn filter_messages_for_report(
    messages: Vec<UnifiedMessage>,
    options: &ReportOptions,
) -> Vec<UnifiedMessage> {
    let mut filtered = messages;

    if let Some(year) = &options.year {
        let year_prefix = format!("{}-", year);
        filtered.retain(|m| m.date.starts_with(&year_prefix));
    }

    if let Some(since) = &options.since {
        filtered.retain(|m| m.date.as_str() >= since.as_str());
    }

    if let Some(until) = &options.until {
        filtered.retain(|m| m.date.as_str() <= until.as_str());
    }

    filtered
}

fn is_headless_path(path: &Path, headless_roots: &[PathBuf]) -> bool {
    headless_roots.iter().any(|root| path.starts_with(root))
}

fn apply_headless_agent(message: &mut UnifiedMessage, is_headless: bool) {
    if is_headless && message.agent.is_none() {
        message.agent = Some("headless".to_string());
    }
}

pub fn parse_local_sources(options: LocalParseOptions) -> Result<ParsedMessages, String> {
    let start = Instant::now();

    let home_dir = get_home_dir_string(&options.home_dir)?;

    let sources = options.sources.clone().unwrap_or_else(|| {
        vec![
            "opencode".to_string(),
            "claude".to_string(),
            "codex".to_string(),
            "gemini".to_string(),
            "amp".to_string(),
            "droid".to_string(),
            "openclaw".to_string(),
        ]
    });

    let local_sources: Vec<String> = sources.into_iter().filter(|s| s != "cursor").collect();

    let scan_result = scanner::scan_all_sources(&home_dir, &local_sources);
    let headless_roots = scanner::headless_roots(&home_dir);

    let mut messages: Vec<ParsedMessage> = Vec::new();

    let opencode_msgs: Vec<ParsedMessage> = scan_result
        .opencode_files
        .par_iter()
        .filter_map(|path| {
            let msg = sessions::opencode::parse_opencode_file(path)?;
            Some(unified_to_parsed(&msg))
        })
        .collect();
    let opencode_count = opencode_msgs.len() as i32;
    messages.extend(opencode_msgs);

    let claude_msgs_raw: Vec<(String, ParsedMessage)> = scan_result
        .claude_files
        .par_iter()
        .flat_map(|path| {
            sessions::claudecode::parse_claude_file(path)
                .into_iter()
                .map(|msg| {
                    let dedup_key = msg.dedup_key.clone().unwrap_or_default();
                    (dedup_key, unified_to_parsed(&msg))
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let mut seen_keys: HashSet<String> = HashSet::new();
    let claude_msgs: Vec<ParsedMessage> = claude_msgs_raw
        .into_iter()
        .filter(|(key, _)| key.is_empty() || seen_keys.insert(key.clone()))
        .map(|(_, msg)| msg)
        .collect();
    let claude_count = claude_msgs.len() as i32;
    messages.extend(claude_msgs);

    let codex_msgs: Vec<ParsedMessage> = scan_result
        .codex_files
        .par_iter()
        .flat_map(|path| {
            let is_headless = is_headless_path(path, &headless_roots);
            sessions::codex::parse_codex_file(path)
                .into_iter()
                .map(|mut msg| {
                    apply_headless_agent(&mut msg, is_headless);
                    unified_to_parsed(&msg)
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let codex_count = codex_msgs.len() as i32;
    messages.extend(codex_msgs);

    let gemini_msgs: Vec<ParsedMessage> = scan_result
        .gemini_files
        .par_iter()
        .flat_map(|path| {
            sessions::gemini::parse_gemini_file(path)
                .into_iter()
                .map(|msg| unified_to_parsed(&msg))
                .collect::<Vec<_>>()
        })
        .collect();
    let gemini_count = gemini_msgs.len() as i32;
    messages.extend(gemini_msgs);

    let amp_msgs: Vec<ParsedMessage> = scan_result
        .amp_files
        .par_iter()
        .flat_map(|path| {
            sessions::amp::parse_amp_file(path)
                .into_iter()
                .map(|msg| unified_to_parsed(&msg))
                .collect::<Vec<_>>()
        })
        .collect();
    let amp_count = amp_msgs.len() as i32;
    messages.extend(amp_msgs);

    let droid_msgs: Vec<ParsedMessage> = scan_result
        .droid_files
        .par_iter()
        .flat_map(|path| {
            sessions::droid::parse_droid_file(path)
                .into_iter()
                .map(|msg| unified_to_parsed(&msg))
                .collect::<Vec<_>>()
        })
        .collect();
    let droid_count = droid_msgs.len() as i32;
    messages.extend(droid_msgs);

    let openclaw_msgs: Vec<ParsedMessage> = scan_result
        .openclaw_files
        .par_iter()
        .flat_map(|path| {
            sessions::openclaw::parse_openclaw_index(path)
                .into_iter()
                .map(|msg| unified_to_parsed(&msg))
                .collect::<Vec<_>>()
        })
        .collect();
    let openclaw_count = openclaw_msgs.len() as i32;
    messages.extend(openclaw_msgs);

    let filtered = filter_parsed_messages(messages, &options);

    Ok(ParsedMessages {
        messages: filtered,
        opencode_count,
        claude_count,
        codex_count,
        gemini_count,
        amp_count,
        droid_count,
        openclaw_count,
        processing_time_ms: start.elapsed().as_millis() as u32,
    })
}

fn unified_to_parsed(msg: &UnifiedMessage) -> ParsedMessage {
    ParsedMessage {
        source: msg.source.clone(),
        model_id: msg.model_id.clone(),
        provider_id: msg.provider_id.clone(),
        session_id: msg.session_id.clone(),
        timestamp: msg.timestamp,
        date: msg.date.clone(),
        input: msg.tokens.input,
        output: msg.tokens.output,
        cache_read: msg.tokens.cache_read,
        cache_write: msg.tokens.cache_write,
        reasoning: msg.tokens.reasoning,
        agent: msg.agent.clone(),
    }
}

fn filter_parsed_messages(
    messages: Vec<ParsedMessage>,
    options: &LocalParseOptions,
) -> Vec<ParsedMessage> {
    let mut filtered = messages;

    if let Some(year) = &options.year {
        let year_prefix = format!("{}-", year);
        filtered.retain(|m| m.date.starts_with(&year_prefix));
    }

    if let Some(since) = &options.since {
        filtered.retain(|m| m.date.as_str() >= since.as_str());
    }

    if let Some(until) = &options.until {
        filtered.retain(|m| m.date.as_str() <= until.as_str());
    }

    filtered
}

pub fn parsed_to_unified(msg: &ParsedMessage, cost: f64) -> UnifiedMessage {
    UnifiedMessage {
        source: msg.source.clone(),
        model_id: msg.model_id.clone(),
        provider_id: msg.provider_id.clone(),
        session_id: msg.session_id.clone(),
        timestamp: msg.timestamp,
        date: msg.date.clone(),
        tokens: TokenBreakdown {
            input: msg.input,
            output: msg.output,
            cache_read: msg.cache_read,
            cache_write: msg.cache_write,
            reasoning: msg.reasoning,
        },
        cost,
        agent: msg.agent.clone(),
        dedup_key: None,
    }
}
