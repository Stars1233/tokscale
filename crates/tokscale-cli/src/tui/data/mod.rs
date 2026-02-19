use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use rayon::prelude::*;
use tokio::runtime::Runtime;

use tokscale_core::pricing::PricingService;
use tokscale_core::sessions::UnifiedMessage;
use tokscale_core::{normalize_model_for_grouping, scanner, sessions};

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
    pub client: String,
    pub tokens: TokenBreakdown,
    pub cost: f64,
    pub session_count: u32,
}

#[derive(Debug, Clone)]
pub struct DailyModelInfo {
    pub client: String,
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
pub enum Client {
    OpenCode,
    Claude,
    Codex,
    Cursor,
    Gemini,
    Amp,
    Droid,
    OpenClaw,
    Pi,
    Kimi,
}

impl Client {
    pub fn all() -> &'static [Client] {
        &[
            Client::OpenCode,
            Client::Claude,
            Client::Codex,
            Client::Cursor,
            Client::Gemini,
            Client::Amp,
            Client::Droid,
            Client::OpenClaw,
            Client::Pi,
            Client::Kimi,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Client::OpenCode => "OpenCode",
            Client::Claude => "Claude",
            Client::Codex => "Codex",
            Client::Cursor => "Cursor",
            Client::Gemini => "Gemini",
            Client::Amp => "Amp",
            Client::Droid => "Droid",
            Client::OpenClaw => "OpenClaw",
            Client::Pi => "Pi",
            Client::Kimi => "Kimi",
        }
    }

    pub fn key(&self) -> char {
        match self {
            Client::OpenCode => '1',
            Client::Claude => '2',
            Client::Codex => '3',
            Client::Cursor => '4',
            Client::Gemini => '5',
            Client::Amp => '6',
            Client::Droid => '7',
            Client::OpenClaw => '8',
            Client::Pi => '9',
            Client::Kimi => '0',
        }
    }

    pub fn from_key(key: char) -> Option<Client> {
        match key {
            '1' => Some(Client::OpenCode),
            '2' => Some(Client::Claude),
            '3' => Some(Client::Codex),
            '4' => Some(Client::Cursor),
            '5' => Some(Client::Gemini),
            '6' => Some(Client::Amp),
            '7' => Some(Client::Droid),
            '8' => Some(Client::OpenClaw),
            '9' => Some(Client::Pi),
            '0' => Some(Client::Kimi),
            _ => None,
        }
    }

    fn to_core_client(self) -> &'static str {
        match self {
            Client::OpenCode => "opencode",
            Client::Claude => "claude",
            Client::Codex => "codex",
            Client::Cursor => "cursor",
            Client::Gemini => "gemini",
            Client::Amp => "amp",
            Client::Droid => "droid",
            Client::OpenClaw => "openclaw",
            Client::Pi => "pi",
            Client::Kimi => "kimi",
        }
    }
}

pub struct DataLoader {
    _sessions_path: Option<PathBuf>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub year: Option<String>,
}

impl DataLoader {
    pub fn new(sessions_path: Option<PathBuf>) -> Self {
        Self {
            _sessions_path: sessions_path,
            since: None,
            until: None,
            year: None,
        }
    }

    pub fn with_filters(
        sessions_path: Option<PathBuf>,
        since: Option<String>,
        until: Option<String>,
        year: Option<String>,
    ) -> Self {
        Self {
            _sessions_path: sessions_path,
            since,
            until,
            year,
        }
    }

    pub fn load(&self, enabled_clients: &[Client]) -> Result<UsageData> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .to_string_lossy()
            .to_string();

        let rt = Runtime::new()?;
        let pricing_result = rt.block_on(async { PricingService::get_or_init().await });
        let pricing = pricing_result.as_ref().ok();

        let sources: Vec<String> = enabled_clients
            .iter()
            .map(|s| s.to_core_client().to_string())
            .collect();

        let scan_result = scanner::scan_all_clients(&home, &sources);

        let mut all_messages: Vec<UnifiedMessage> = Vec::new();

        // OpenCode: read both SQLite (1.2+) and legacy JSON, deduplicate by message ID
        let mut opencode_seen: HashSet<String> = HashSet::new();

        if enabled_clients.contains(&Client::OpenCode) {
            if let Some(db_path) = &scan_result.opencode_db {
                let sqlite_messages: Vec<UnifiedMessage> =
                    sessions::opencode::parse_opencode_sqlite(db_path);
                for msg in &sqlite_messages {
                    if let Some(ref key) = msg.dedup_key {
                        opencode_seen.insert(key.clone());
                    }
                }
                all_messages.extend(sqlite_messages);
            }

            let json_messages: Vec<UnifiedMessage> = scan_result
                .opencode_files
                .par_iter()
                .filter_map(|path| sessions::opencode::parse_opencode_file(path))
                .collect();
            all_messages.extend(json_messages.into_iter().filter(|msg| {
                msg.dedup_key
                    .as_ref()
                    .map_or(true, |key| opencode_seen.insert(key.clone()))
            }));
        }

        if enabled_clients.contains(&Client::Claude) {
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

        if enabled_clients.contains(&Client::Codex) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .codex_files
                .par_iter()
                .flat_map(|path| sessions::codex::parse_codex_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_clients.contains(&Client::Cursor) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .cursor_files
                .par_iter()
                .flat_map(|path| sessions::cursor::parse_cursor_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_clients.contains(&Client::Gemini) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .gemini_files
                .par_iter()
                .flat_map(|path| sessions::gemini::parse_gemini_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_clients.contains(&Client::Amp) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .amp_files
                .par_iter()
                .flat_map(|path| sessions::amp::parse_amp_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_clients.contains(&Client::Droid) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .droid_files
                .par_iter()
                .flat_map(|path| sessions::droid::parse_droid_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_clients.contains(&Client::OpenClaw) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .openclaw_files
                .par_iter()
                .flat_map(|path| sessions::openclaw::parse_openclaw_transcript(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_clients.contains(&Client::Pi) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .pi_files
                .par_iter()
                .flat_map(|path| sessions::pi::parse_pi_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if enabled_clients.contains(&Client::Kimi) {
            let msgs: Vec<UnifiedMessage> = scan_result
                .kimi_files
                .par_iter()
                .flat_map(|path| sessions::kimi::parse_kimi_file(path))
                .collect();
            all_messages.extend(msgs);
        }

        if let Some(svc) = pricing {
            for msg in &mut all_messages {
                let is_gemini = msg.client.eq_ignore_ascii_case("gemini");
                let calculated_cost = if is_gemini {
                    svc.calculate_cost(
                        &msg.model_id,
                        msg.tokens.input,
                        msg.tokens.output + msg.tokens.reasoning,
                        0,
                        0,
                        0,
                    )
                } else {
                    svc.calculate_cost(
                        &msg.model_id,
                        msg.tokens.input,
                        msg.tokens.output,
                        msg.tokens.cache_read,
                        msg.tokens.cache_write,
                        msg.tokens.reasoning,
                    )
                };
                if calculated_cost > 0.0 {
                    msg.cost = calculated_cost;
                }
            }
        }

        // Apply date filters if specified
        let filtered_messages = self.apply_date_filters(all_messages);

        self.aggregate_messages(filtered_messages)
    }

    fn aggregate_messages(&self, messages: Vec<UnifiedMessage>) -> Result<UsageData> {
        let mut model_map: HashMap<String, ModelUsage> = HashMap::new();
        let mut daily_map: HashMap<NaiveDate, DailyUsage> = HashMap::new();
        let mut model_session_ids: HashMap<String, HashSet<String>> = HashMap::new();

        for msg in &messages {
            let normalized_model = normalize_model_for_grouping(&msg.model_id);
            let key = format!("{}:{}:{}", msg.client, msg.provider_id, normalized_model);

            let model_entry = model_map.entry(key.clone()).or_insert_with(|| ModelUsage {
                model: normalized_model.clone(),
                provider: msg.provider_id.clone(),
                client: msg.client.clone(),
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

            let session_key = format!("{}:{}", msg.client, msg.session_id);
            let model_sessions = model_session_ids.entry(key).or_insert_with(HashSet::new);
            if model_sessions.insert(session_key) {
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
                    .entry(normalized_model.clone())
                    .or_insert_with(|| DailyModelInfo {
                        client: msg.client.clone(),
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
                .then_with(|| a.client.cmp(&b.client))
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

    fn apply_date_filters(&self, messages: Vec<UnifiedMessage>) -> Vec<UnifiedMessage> {
        // If no filters are specified, return all messages
        if self.since.is_none() && self.until.is_none() && self.year.is_none() {
            return messages;
        }

        messages
            .into_iter()
            .filter(|msg| {
                if let Some(date) = parse_date(&msg.date) {
                    // Check year filter
                    if let Some(ref year_str) = self.year {
                        if let Ok(year) = year_str.parse::<i32>() {
                            if date.year() != year {
                                return false;
                            }
                        }
                    }

                    // Check since filter
                    if let Some(ref since_str) = self.since {
                        if let Some(since_date) = parse_date(since_str) {
                            if date < since_date {
                                return false;
                            }
                        }
                    }

                    // Check until filter
                    if let Some(ref until_str) = self.until {
                        if let Some(until_date) = parse_date(until_str) {
                            if date > until_date {
                                return false;
                            }
                        }
                    }

                    true
                } else {
                    false
                }
            })
            .collect()
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

    let daily_map: HashMap<NaiveDate, &DailyUsage> = daily.iter().map(|d| (d.date, d)).collect();

    let max_cost = daily.iter().map(|d| d.cost).fold(0.0_f64, |a, b| a.max(b));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_all() {
        let clients = Client::all();
        assert_eq!(clients.len(), 10);
        assert_eq!(clients[0], Client::OpenCode);
        assert_eq!(clients[1], Client::Claude);
        assert_eq!(clients[2], Client::Codex);
        assert_eq!(clients[3], Client::Cursor);
        assert_eq!(clients[4], Client::Gemini);
        assert_eq!(clients[5], Client::Amp);
        assert_eq!(clients[6], Client::Droid);
        assert_eq!(clients[7], Client::OpenClaw);
        assert_eq!(clients[8], Client::Pi);
        assert_eq!(clients[9], Client::Kimi);
    }

    #[test]
    fn test_client_as_str() {
        assert_eq!(Client::OpenCode.as_str(), "OpenCode");
        assert_eq!(Client::Claude.as_str(), "Claude");
        assert_eq!(Client::Codex.as_str(), "Codex");
        assert_eq!(Client::Cursor.as_str(), "Cursor");
        assert_eq!(Client::Gemini.as_str(), "Gemini");
        assert_eq!(Client::Amp.as_str(), "Amp");
        assert_eq!(Client::Droid.as_str(), "Droid");
        assert_eq!(Client::OpenClaw.as_str(), "OpenClaw");
        assert_eq!(Client::Pi.as_str(), "Pi");
        assert_eq!(Client::Kimi.as_str(), "Kimi");
    }

    #[test]
    fn test_client_key() {
        assert_eq!(Client::OpenCode.key(), '1');
        assert_eq!(Client::Claude.key(), '2');
        assert_eq!(Client::Codex.key(), '3');
        assert_eq!(Client::Cursor.key(), '4');
        assert_eq!(Client::Gemini.key(), '5');
        assert_eq!(Client::Amp.key(), '6');
        assert_eq!(Client::Droid.key(), '7');
        assert_eq!(Client::OpenClaw.key(), '8');
        assert_eq!(Client::Pi.key(), '9');
        assert_eq!(Client::Kimi.key(), '0');
    }

    #[test]
    fn test_client_from_key() {
        assert_eq!(Client::from_key('1'), Some(Client::OpenCode));
        assert_eq!(Client::from_key('2'), Some(Client::Claude));
        assert_eq!(Client::from_key('3'), Some(Client::Codex));
        assert_eq!(Client::from_key('4'), Some(Client::Cursor));
        assert_eq!(Client::from_key('5'), Some(Client::Gemini));
        assert_eq!(Client::from_key('6'), Some(Client::Amp));
        assert_eq!(Client::from_key('7'), Some(Client::Droid));
        assert_eq!(Client::from_key('8'), Some(Client::OpenClaw));
        assert_eq!(Client::from_key('9'), Some(Client::Pi));
        assert_eq!(Client::from_key('0'), Some(Client::Kimi));
        assert_eq!(Client::from_key('a'), None);
    }

    #[test]
    fn test_token_breakdown_total() {
        let breakdown = TokenBreakdown {
            input: 100,
            output: 200,
            cache_read: 50,
            cache_write: 25,
            reasoning: 10,
        };
        assert_eq!(breakdown.total(), 385);
    }

    #[test]
    fn test_token_breakdown_total_with_overflow() {
        let breakdown = TokenBreakdown {
            input: u64::MAX,
            output: 1,
            cache_read: 0,
            cache_write: 0,
            reasoning: 0,
        };
        // saturating_add should prevent overflow
        assert_eq!(breakdown.total(), u64::MAX);
    }

    #[test]
    fn test_token_breakdown_default() {
        let breakdown = TokenBreakdown::default();
        assert_eq!(breakdown.input, 0);
        assert_eq!(breakdown.output, 0);
        assert_eq!(breakdown.cache_read, 0);
        assert_eq!(breakdown.cache_write, 0);
        assert_eq!(breakdown.reasoning, 0);
        assert_eq!(breakdown.total(), 0);
    }

    #[test]
    fn test_data_loader_new() {
        let loader = DataLoader::new(None);
        assert!(loader._sessions_path.is_none());
        assert!(loader.since.is_none());
        assert!(loader.until.is_none());
        assert!(loader.year.is_none());
    }

    #[test]
    fn test_data_loader_with_filters() {
        let loader = DataLoader::with_filters(
            Some(PathBuf::from("/tmp/sessions")),
            Some("2024-01-01".to_string()),
            Some("2024-12-31".to_string()),
            Some("2024".to_string()),
        );
        
        assert_eq!(loader._sessions_path, Some(PathBuf::from("/tmp/sessions")));
        assert_eq!(loader.since, Some("2024-01-01".to_string()));
        assert_eq!(loader.until, Some("2024-12-31".to_string()));
        assert_eq!(loader.year, Some("2024".to_string()));
    }

    #[test]
    fn test_parse_date() {
        assert_eq!(
            parse_date("2024-01-15"),
            Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        );
        assert_eq!(
            parse_date("2024-12-31"),
            Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
        );
        assert_eq!(parse_date("invalid"), None);
        assert_eq!(parse_date("2024-13-01"), None);
        assert_eq!(parse_date(""), None);
    }
}
