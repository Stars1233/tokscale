use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use rayon::prelude::*;
use tokio::runtime::Runtime;

use tokscale_core::pricing::PricingService;
use tokscale_core::sessions::UnifiedMessage;
use tokscale_core::{scanner, sessions};

#[derive(Debug, Clone, Default)]
pub struct TokenBreakdown {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub reasoning: u64,
}

impl TokenBreakdown {
    pub fn total(&self) -> u64 {
        self.input
            .saturating_add(self.output)
            .saturating_add(self.cache_read)
            .saturating_add(self.cache_write)
            .saturating_add(self.reasoning)
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

    fn to_core_source(self) -> &'static str {
        match self {
            Source::OpenCode => "opencode",
            Source::Claude => "claude",
            Source::Codex => "codex",
            Source::Cursor => "cursor",
            Source::Gemini => "gemini",
            Source::Amp => "amp",
            Source::Droid => "droid",
            Source::OpenClaw => "openclaw",
        }
    }
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

    pub fn load(&self, enabled_sources: &[Source]) -> Result<UsageData> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .to_string_lossy()
            .to_string();

        let rt = Runtime::new()?;
        let pricing_result = rt.block_on(async { PricingService::get_or_init().await });
        let pricing = pricing_result.as_ref().ok();

        let sources: Vec<String> = enabled_sources
            .iter()
            .map(|s| s.to_core_source().to_string())
            .collect();

        let scan_result = scanner::scan_all_sources(&home, &sources);

        let mut all_messages: Vec<UnifiedMessage> = Vec::new();

        if enabled_sources.contains(&Source::OpenCode) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .opencode_files
                .par_iter()
                .filter_map(|path| sessions::opencode::parse_opencode_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_sources.contains(&Source::Claude) {
            let msgs_raw: Vec<UnifiedMessage> = scan_result
                .claude_files
                .par_iter()
                .flat_map(|path| sessions::claudecode::parse_claude_file(path))
                .collect();

            let mut seen_keys: HashSet<String> = HashSet::new();
            let msgs: Vec<UnifiedMessage> = msgs_raw
                .into_iter()
                .filter(|m| {
                    m.dedup_key
                        .as_ref()
                        .is_none_or(|k| k.is_empty() || seen_keys.insert(k.clone()))
                })
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_sources.contains(&Source::Codex) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .codex_files
                .par_iter()
                .flat_map(|path| sessions::codex::parse_codex_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_sources.contains(&Source::Cursor) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .cursor_files
                .par_iter()
                .flat_map(|path| sessions::cursor::parse_cursor_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_sources.contains(&Source::Gemini) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .gemini_files
                .par_iter()
                .flat_map(|path| sessions::gemini::parse_gemini_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_sources.contains(&Source::Amp) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .amp_files
                .par_iter()
                .flat_map(|path| sessions::amp::parse_amp_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_sources.contains(&Source::Droid) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .droid_files
                .par_iter()
                .flat_map(|path| sessions::droid::parse_droid_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_sources.contains(&Source::OpenClaw) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .openclaw_files
                .par_iter()
                .flat_map(|path| sessions::openclaw::parse_openclaw_index(path))
                .collect();
            all_messages.extend(msgs);
        }

        if let Some(svc) = pricing {
            for msg in &mut all_messages {
                let calculated_cost = svc.calculate_cost(
                    &msg.model_id,
                    msg.tokens.input,
                    msg.tokens.output,
                    msg.tokens.cache_read,
                    msg.tokens.cache_write,
                    msg.tokens.reasoning,
                );
                // Only overwrite cost if pricing service returns a positive value
                // Preserve original cost for sources like cursor/amp that provide pre-calculated costs
                if calculated_cost > 0.0 {
                    msg.cost = calculated_cost;
                }
            }
        }

        self.aggregate_messages(all_messages)
    }

    fn aggregate_messages(&self, messages: Vec<UnifiedMessage>) -> Result<UsageData> {
        let mut model_map: HashMap<String, ModelUsage> = HashMap::new();
        let mut daily_map: HashMap<NaiveDate, DailyUsage> = HashMap::new();
        let mut session_ids: HashSet<String> = HashSet::new();

        for msg in &messages {
            let key = format!("{}:{}:{}", msg.source, msg.provider_id, msg.model_id);

            let model_entry = model_map.entry(key.clone()).or_insert_with(|| ModelUsage {
                model: msg.model_id.clone(),
                provider: msg.provider_id.clone(),
                source: msg.source.clone(),
                tokens: TokenBreakdown::default(),
                cost: 0.0,
                session_count: 0,
            });

            model_entry.tokens.input = model_entry
                .tokens
                .input
                .saturating_add(msg.tokens.input.max(0) as u64);
            model_entry.tokens.output = model_entry
                .tokens
                .output
                .saturating_add(msg.tokens.output.max(0) as u64);
            model_entry.tokens.cache_read = model_entry
                .tokens
                .cache_read
                .saturating_add(msg.tokens.cache_read.max(0) as u64);
            model_entry.tokens.cache_write = model_entry
                .tokens
                .cache_write
                .saturating_add(msg.tokens.cache_write.max(0) as u64);
            model_entry.tokens.reasoning = model_entry
                .tokens
                .reasoning
                .saturating_add(msg.tokens.reasoning.max(0) as u64);
            let msg_cost = if msg.cost.is_finite() && msg.cost >= 0.0 {
                msg.cost
            } else {
                0.0
            };
            model_entry.cost += msg_cost;

            let session_key = format!("{}:{}", msg.source, msg.session_id);
            if session_ids.insert(session_key) {
                model_entry.session_count += 1;
            }

            if let Some(date) = parse_date(&msg.date) {
                let daily_entry = daily_map.entry(date).or_insert_with(|| DailyUsage {
                    date,
                    tokens: TokenBreakdown::default(),
                    cost: 0.0,
                    models: HashMap::new(),
                });

                daily_entry.tokens.input = daily_entry
                    .tokens
                    .input
                    .saturating_add(msg.tokens.input.max(0) as u64);
                daily_entry.tokens.output = daily_entry
                    .tokens
                    .output
                    .saturating_add(msg.tokens.output.max(0) as u64);
                daily_entry.tokens.cache_read = daily_entry
                    .tokens
                    .cache_read
                    .saturating_add(msg.tokens.cache_read.max(0) as u64);
                daily_entry.tokens.cache_write = daily_entry
                    .tokens
                    .cache_write
                    .saturating_add(msg.tokens.cache_write.max(0) as u64);
                daily_entry.tokens.reasoning = daily_entry
                    .tokens
                    .reasoning
                    .saturating_add(msg.tokens.reasoning.max(0) as u64);
                let msg_cost = if msg.cost.is_finite() && msg.cost >= 0.0 {
                    msg.cost
                } else {
                    0.0
                };
                daily_entry.cost += msg_cost;

                let model_info = daily_entry
                    .models
                    .entry(msg.model_id.clone())
                    .or_insert_with(|| DailyModelInfo {
                        source: msg.source.clone(),
                        tokens: TokenBreakdown::default(),
                        cost: 0.0,
                    });

                model_info.tokens.input = model_info
                    .tokens
                    .input
                    .saturating_add(msg.tokens.input.max(0) as u64);
                model_info.tokens.output = model_info
                    .tokens
                    .output
                    .saturating_add(msg.tokens.output.max(0) as u64);
                model_info.tokens.cache_read = model_info
                    .tokens
                    .cache_read
                    .saturating_add(msg.tokens.cache_read.max(0) as u64);
                model_info.tokens.cache_write = model_info
                    .tokens
                    .cache_write
                    .saturating_add(msg.tokens.cache_write.max(0) as u64);
                model_info.tokens.reasoning = model_info
                    .tokens
                    .reasoning
                    .saturating_add(msg.tokens.reasoning.max(0) as u64);
                let model_msg_cost = if msg.cost.is_finite() && msg.cost >= 0.0 {
                    msg.cost
                } else {
                    0.0
                };
                model_info.cost += model_msg_cost;
            }
        }

        let mut models: Vec<ModelUsage> = model_map.into_values().collect();
        models.sort_by(|a, b| {
            b.cost
                .total_cmp(&a.cost)
                .then_with(|| a.model.cmp(&b.model))
                .then_with(|| a.provider.cmp(&b.provider))
                .then_with(|| a.source.cmp(&b.source))
        });

        let mut daily: Vec<DailyUsage> = daily_map.into_values().collect();
        daily.sort_by(|a, b| b.date.cmp(&a.date));

        let total_tokens: u64 = models.iter().map(|m| m.tokens.total()).sum();
        let total_cost: f64 = models
            .iter()
            .map(|m| if m.cost.is_finite() { m.cost } else { 0.0 })
            .sum();

        let graph = build_contribution_graph(&daily);
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

fn parse_date(date_str: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
}

fn build_contribution_graph(daily: &[DailyUsage]) -> GraphData {
    if daily.is_empty() {
        return GraphData { weeks: vec![] };
    }

    let today = Utc::now().date_naive();
    let days_to_sunday = today.weekday().num_days_from_sunday();
    let end_date = today;
    let start_date = end_date - chrono::Duration::days(364 + days_to_sunday as i64);

    let daily_map: HashMap<NaiveDate, &DailyUsage> =
        daily.iter().map(|d| (d.date, d)).collect();

    let max_cost = daily
        .iter()
        .map(|d| d.cost)
        .fold(0.0_f64, |a, b| a.max(b));

    let mut weeks: Vec<Vec<Option<ContributionDay>>> = Vec::new();
    let mut current_week: Vec<Option<ContributionDay>> = Vec::new();

    let mut current_date = start_date;
    while current_date <= end_date {
        let day = if let Some(usage) = daily_map.get(&current_date) {
            let raw_intensity = if max_cost > 0.0 {
                usage.cost / max_cost
            } else {
                0.0
            };
            let intensity = if raw_intensity.is_finite() {
                raw_intensity.clamp(0.0, 1.0)
            } else {
                0.0
            };
            Some(ContributionDay {
                date: current_date,
                tokens: usage.tokens.total(),
                cost: usage.cost,
                intensity,
            })
        } else {
            Some(ContributionDay {
                date: current_date,
                tokens: 0,
                cost: 0.0,
                intensity: 0.0,
            })
        };

        current_week.push(day);

        if current_date.weekday() == chrono::Weekday::Sat || current_date == end_date {
            weeks.push(current_week);
            current_week = Vec::new();
        }

        current_date += chrono::Duration::days(1);
    }

    GraphData { weeks }
}

fn calculate_streaks(daily: &[DailyUsage]) -> (u32, u32) {
    if daily.is_empty() {
        return (0, 0);
    }

    let today = Utc::now().date_naive();
    let dates: HashSet<NaiveDate> = daily.iter().map(|d| d.date).collect();

    let mut current_streak = 0u32;
    let mut check_date = today;

    while dates.contains(&check_date) {
        current_streak += 1;
        check_date -= chrono::Duration::days(1);
    }

    if current_streak == 0 {
        let yesterday = today - chrono::Duration::days(1);
        check_date = yesterday;
        while dates.contains(&check_date) {
            current_streak += 1;
            check_date -= chrono::Duration::days(1);
        }
    }

    let mut longest_streak = 0u32;
    let mut sorted_dates: Vec<NaiveDate> = dates.into_iter().collect();
    sorted_dates.sort();

    let mut streak = 0u32;
    let mut prev_date: Option<NaiveDate> = None;

    for date in sorted_dates {
        if let Some(prev) = prev_date {
            if date == prev + chrono::Duration::days(1) {
                streak += 1;
            } else {
                longest_streak = longest_streak.max(streak);
                streak = 1;
            }
        } else {
            streak = 1;
        }
        prev_date = Some(date);
    }
    longest_streak = longest_streak.max(streak);

    (current_streak, longest_streak)
}
