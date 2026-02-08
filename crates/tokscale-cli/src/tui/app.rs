use std::collections::HashSet;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use super::data::{DailyUsage, DataLoader, ModelUsage, Source, UsageData};
use super::settings::Settings;
use super::themes::{Theme, ThemeName};

/// Configuration for TUI initialization
pub struct TuiConfig {
    pub theme: String,
    pub refresh: u64,
    pub sessions_path: Option<String>,
    pub sources: Option<Vec<String>>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub year: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Models,
    Daily,
    Stats,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[Tab::Overview, Tab::Models, Tab::Daily, Tab::Stats]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Models => "Models",
            Tab::Daily => "Daily",
            Tab::Stats => "Stats",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Tab::Overview => "Ovw",
            Tab::Models => "Mod",
            Tab::Daily => "Day",
            Tab::Stats => "Sta",
        }
    }

    pub fn next(self) -> Tab {
        match self {
            Tab::Overview => Tab::Models,
            Tab::Models => Tab::Daily,
            Tab::Daily => Tab::Stats,
            Tab::Stats => Tab::Overview,
        }
    }

    pub fn prev(self) -> Tab {
        match self {
            Tab::Overview => Tab::Stats,
            Tab::Models => Tab::Overview,
            Tab::Daily => Tab::Models,
            Tab::Stats => Tab::Daily,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Cost,
    Tokens,
    Date,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub struct ClickArea {
    pub rect: Rect,
    pub action: ClickAction,
}

#[derive(Debug, Clone)]
pub enum ClickAction {
    Tab(Tab),
    Source(Source),
    Sort(SortField),
    GraphCell { week: usize, day: usize },
}

pub struct App {
    pub should_quit: bool,
    pub current_tab: Tab,
    pub theme: Theme,
    pub settings: Settings,
    pub data: UsageData,
    pub data_loader: DataLoader,

    pub enabled_sources: HashSet<Source>,
    pub sort_field: SortField,
    pub sort_direction: SortDirection,

    pub scroll_offset: usize,
    pub selected_index: usize,
    pub max_visible_items: usize,

    pub selected_graph_cell: Option<(usize, usize)>,

    pub auto_refresh: bool,
    pub auto_refresh_interval: Duration,
    pub last_refresh: Instant,

    pub status_message: Option<String>,
    pub status_message_time: Option<Instant>,

    pub terminal_width: u16,
    pub terminal_height: u16,

    pub click_areas: Vec<ClickArea>,

    pub spinner_frame: usize,
}

impl App {
    pub fn new(config: TuiConfig) -> Result<Self> {
        let settings = Settings::load();
        let theme_name: ThemeName = config
            .theme
            .parse()
            .unwrap_or_else(|_| settings.theme_name());
        let theme = Theme::from_name(theme_name);

        let mut enabled_sources = HashSet::new();

        // If sources are specified via CLI, use those
        if let Some(ref cli_sources) = config.sources {
            for source_str in cli_sources {
                // Map source string to Source enum
                match source_str.as_str() {
                    "opencode" => enabled_sources.insert(Source::OpenCode),
                    "claude" => enabled_sources.insert(Source::Claude),
                    "codex" => enabled_sources.insert(Source::Codex),
                    "cursor" => enabled_sources.insert(Source::Cursor),
                    "gemini" => enabled_sources.insert(Source::Gemini),
                    "amp" => enabled_sources.insert(Source::Amp),
                    "droid" => enabled_sources.insert(Source::Droid),
                    "openclaw" => enabled_sources.insert(Source::OpenClaw),
                    _ => false,
                };
            }
        } else {
            // Otherwise use settings
            for source in Source::all() {
                if settings
                    .enabled_sources
                    .contains(&source.as_str().to_string())
                {
                    enabled_sources.insert(*source);
                }
            }
        }

        if enabled_sources.is_empty() {
            for source in Source::all() {
                enabled_sources.insert(*source);
            }
        }

        let auto_refresh_interval = if config.refresh > 0 {
            Duration::from_secs(config.refresh)
        } else if settings.auto_refresh_interval > 0 {
            Duration::from_secs(settings.auto_refresh_interval)
        } else {
            Duration::from_secs(30)
        };

        let data_loader = DataLoader::with_filters(
            config.sessions_path.map(std::path::PathBuf::from),
            config.since,
            config.until,
            config.year,
        );

        Ok(Self {
            should_quit: false,
            current_tab: Tab::Overview,
            theme,
            settings,
            data: UsageData::default(),
            data_loader,
            enabled_sources,
            sort_field: SortField::Cost,
            sort_direction: SortDirection::Descending,
            scroll_offset: 0,
            selected_index: 0,
            max_visible_items: 20,
            selected_graph_cell: None,
            auto_refresh: config.refresh > 0,
            auto_refresh_interval,
            last_refresh: Instant::now(),
            status_message: None,
            status_message_time: None,
            terminal_width: 80,
            terminal_height: 24,
            click_areas: Vec::new(),
            spinner_frame: 0,
        })
    }

    pub fn load_data(&mut self) -> Result<()> {
        self.data.loading = true;
        let sources: Vec<Source> = self.enabled_sources.iter().copied().collect();
        match self.data_loader.load(&sources) {
            Ok(data) => {
                self.data = data;
                self.last_refresh = Instant::now();
                self.clamp_selection();
                self.set_status("Data loaded");
            }
            Err(e) => {
                self.data.loading = false;
                self.data.error = Some(e.to_string());
                self.set_status(&format!("Error: {}", e));
            }
        }
        Ok(())
    }

    pub fn on_tick(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % 20;

        if let Some(status_time) = self.status_message_time {
            if status_time.elapsed() > Duration::from_secs(3) {
                self.status_message = None;
                self.status_message_time = None;
            }
        }

        if self.auto_refresh && self.last_refresh.elapsed() >= self.auto_refresh_interval {
            let _ = self.load_data();
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return true;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return true;
            }
            KeyCode::Tab => {
                self.current_tab = self.current_tab.next();
                self.reset_selection();
            }
            KeyCode::BackTab => {
                self.current_tab = self.current_tab.prev();
                self.reset_selection();
            }
            KeyCode::Left => {
                self.current_tab = self.current_tab.prev();
                self.reset_selection();
            }
            KeyCode::Right => {
                self.current_tab = self.current_tab.next();
                self.reset_selection();
            }
            KeyCode::Up => {
                self.move_selection_up();
            }
            KeyCode::Down => {
                self.move_selection_down();
            }
            KeyCode::Char('c') => {
                self.set_sort(SortField::Cost);
            }
            KeyCode::Char('t') => {
                self.set_sort(SortField::Tokens);
            }
            KeyCode::Char('d') => {
                self.set_sort(SortField::Date);
            }
            KeyCode::Char('p') => {
                self.cycle_theme();
            }
            KeyCode::Char('r') => {
                let _ = self.load_data();
            }
            KeyCode::Char('R') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.toggle_auto_refresh();
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.increase_refresh_interval();
            }
            KeyCode::Char('-') => {
                self.decrease_refresh_interval();
            }
            KeyCode::Char('y') => {
                self.copy_selected_to_clipboard();
            }
            KeyCode::Char('e') => {
                self.export_to_json();
            }
            KeyCode::Char(c @ '1'..='8') => {
                self.toggle_source(c);
            }
            KeyCode::Enter => {
                if self.current_tab == Tab::Stats {
                    self.handle_graph_selection();
                }
            }
            KeyCode::Esc => {
                self.selected_graph_cell = None;
            }
            _ => {}
        }
        false
    }

    pub fn handle_mouse_event(&mut self, event: MouseEvent) {
        if let MouseEventKind::Down(MouseButton::Left) = event.kind {
            let x = event.column;
            let y = event.row;

            for area in &self.click_areas {
                if x >= area.rect.x
                    && x < area.rect.x + area.rect.width
                    && y >= area.rect.y
                    && y < area.rect.y + area.rect.height
                {
                    match &area.action {
                        ClickAction::Tab(tab) => {
                            self.current_tab = *tab;
                            self.reset_selection();
                        }
                        ClickAction::Source(source) => {
                            self.toggle_source(source.key());
                        }
                        ClickAction::Sort(field) => {
                            self.set_sort(*field);
                        }
                        ClickAction::GraphCell { week, day } => {
                            self.selected_graph_cell = Some((*week, *day));
                        }
                    }
                    break;
                }
            }
        }
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;
        // Ensure at least 1 visible item to prevent division/slice issues
        self.max_visible_items = (height.saturating_sub(10) as usize).max(1);
        self.clamp_selection();
    }

    /// Clamp selection and scroll offset to valid bounds after data/resize changes
    fn clamp_selection(&mut self) {
        let len = self.get_current_list_len();
        if len == 0 {
            self.selected_index = 0;
            self.scroll_offset = 0;
            return;
        }
        self.selected_index = self.selected_index.min(len.saturating_sub(1));
        let max_scroll = len.saturating_sub(self.max_visible_items);
        self.scroll_offset = self.scroll_offset.min(max_scroll);
    }

    pub fn clear_click_areas(&mut self) {
        self.click_areas.clear();
    }

    pub fn add_click_area(&mut self, rect: Rect, action: ClickAction) {
        self.click_areas.push(ClickArea { rect, action });
    }

    fn reset_selection(&mut self) {
        self.scroll_offset = 0;
        self.selected_index = 0;
        self.selected_graph_cell = None;
    }

    fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }

    fn move_selection_down(&mut self) {
        let max_index = self.get_current_list_len().saturating_sub(1);
        if self.selected_index < max_index {
            self.selected_index += 1;
            if self.selected_index >= self.scroll_offset + self.max_visible_items {
                self.scroll_offset = self.selected_index - self.max_visible_items + 1;
            }
        }
    }

    fn get_current_list_len(&self) -> usize {
        match self.current_tab {
            Tab::Overview | Tab::Models => self.data.models.len(),
            Tab::Daily => self.data.daily.len(),
            Tab::Stats => 0,
        }
    }

    fn set_sort(&mut self, field: SortField) {
        if self.sort_field == field {
            self.sort_direction = match self.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            self.sort_field = field;
            self.sort_direction = SortDirection::Descending;
        }
        self.reset_selection();
        self.set_status(&format!(
            "Sorted by {:?} {:?}",
            self.sort_field, self.sort_direction
        ));
    }

    fn cycle_theme(&mut self) {
        let new_theme = self.theme.name.next();
        self.theme = Theme::from_name(new_theme);
        self.settings.set_theme(new_theme);
        if let Err(e) = self.settings.save() {
            self.set_status(&format!(
                "Theme: {} (save failed: {})",
                new_theme.as_str(),
                e
            ));
        } else {
            self.set_status(&format!("Theme: {}", new_theme.as_str()));
        }
    }

    fn toggle_source(&mut self, key: char) {
        if let Some(source) = Source::from_key(key) {
            if self.enabled_sources.contains(&source) {
                if self.enabled_sources.len() > 1 {
                    self.enabled_sources.remove(&source);
                    self.set_status(&format!("Disabled {}", source.as_str()));
                }
            } else {
                self.enabled_sources.insert(source);
                self.set_status(&format!("Enabled {}", source.as_str()));
            }
            self.update_settings_sources();
            let _ = self.load_data();
        }
    }

    fn update_settings_sources(&mut self) {
        self.settings.enabled_sources = self
            .enabled_sources
            .iter()
            .map(|s| s.as_str().to_string())
            .collect();
        if let Err(e) = self.settings.save() {
            self.set_status(&format!("Settings save failed: {}", e));
        }
    }

    fn toggle_auto_refresh(&mut self) {
        self.auto_refresh = !self.auto_refresh;
        if self.auto_refresh {
            self.set_status(&format!(
                "Auto-refresh ON ({}s)",
                self.auto_refresh_interval.as_secs()
            ));
        } else {
            self.set_status("Auto-refresh OFF");
        }
    }

    fn increase_refresh_interval(&mut self) {
        let secs = self.auto_refresh_interval.as_secs();
        self.auto_refresh_interval = Duration::from_secs(secs.saturating_add(10).min(300));
        self.settings.auto_refresh_interval = self.auto_refresh_interval.as_secs();
        let save_result = self.settings.save();
        let msg = format!(
            "Refresh interval: {}s",
            self.auto_refresh_interval.as_secs()
        );
        if let Err(e) = save_result {
            self.set_status(&format!("{} (save failed: {})", msg, e));
        } else {
            self.set_status(&msg);
        }
    }

    fn decrease_refresh_interval(&mut self) {
        let secs = self.auto_refresh_interval.as_secs();
        self.auto_refresh_interval = Duration::from_secs(secs.saturating_sub(10).max(10));
        self.settings.auto_refresh_interval = self.auto_refresh_interval.as_secs();
        let save_result = self.settings.save();
        let msg = format!(
            "Refresh interval: {}s",
            self.auto_refresh_interval.as_secs()
        );
        if let Err(e) = save_result {
            self.set_status(&format!("{} (save failed: {})", msg, e));
        } else {
            self.set_status(&msg);
        }
    }

    fn copy_selected_to_clipboard(&mut self) {
        let text = match self.current_tab {
            Tab::Overview | Tab::Models => self
                .get_sorted_models()
                .get(self.selected_index)
                .map(|m| format!("{}: {} tokens, ${:.4}", m.model, m.tokens.total(), m.cost)),
            Tab::Daily => self
                .get_sorted_daily()
                .get(self.selected_index)
                .map(|d| format!("{}: {} tokens, ${:.4}", d.date, d.tokens.total(), d.cost)),
            Tab::Stats => None,
        };

        if let Some(text) = text {
            match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(&text)) {
                Ok(_) => self.set_status("Copied to clipboard"),
                Err(_) => self.set_status("Failed to copy"),
            }
        }
    }

    fn export_to_json(&mut self) {
        let export_data = serde_json::json!({
            "models": self.data.models.iter().map(|m| serde_json::json!({
                "model": m.model,
                "provider": m.provider,
                "source": m.source,
                "tokens": {
                    "input": m.tokens.input,
                    "output": m.tokens.output,
                    "cacheRead": m.tokens.cache_read,
                    "cacheWrite": m.tokens.cache_write,
                    "total": m.tokens.total()
                },
                "cost": m.cost,
                "sessionCount": m.session_count
            })).collect::<Vec<_>>(),
            "daily": self.data.daily.iter().map(|d| serde_json::json!({
                "date": d.date.to_string(),
                "tokens": {
                    "input": d.tokens.input,
                    "output": d.tokens.output,
                    "cacheRead": d.tokens.cache_read,
                    "cacheWrite": d.tokens.cache_write,
                    "total": d.tokens.total()
                },
                "cost": d.cost
            })).collect::<Vec<_>>(),
            "totals": {
                "tokens": self.data.total_tokens,
                "cost": self.data.total_cost
            }
        });

        let filename = format!(
            "tokscale-export-{}.json",
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        );

        match serde_json::to_string_pretty(&export_data) {
            Ok(json) => match std::fs::write(&filename, json) {
                Ok(_) => self.set_status(&format!("Exported to {}", filename)),
                Err(e) => self.set_status(&format!("Export failed: {}", e)),
            },
            Err(e) => self.set_status(&format!("Export failed: {}", e)),
        }
    }

    fn handle_graph_selection(&mut self) {
        if self.current_tab == Tab::Stats && self.selected_graph_cell.is_some() {
            self.set_status("Press ESC to deselect");
        }
    }

    pub fn set_status(&mut self, message: &str) {
        self.status_message = Some(message.to_string());
        self.status_message_time = Some(Instant::now());
    }

    pub fn get_sorted_models(&self) -> Vec<&ModelUsage> {
        let mut models: Vec<&ModelUsage> = self.data.models.iter().collect();

        let tie_breaker = |a: &&ModelUsage, b: &&ModelUsage| {
            a.model
                .cmp(&b.model)
                .then_with(|| a.provider.cmp(&b.provider))
                .then_with(|| a.source.cmp(&b.source))
        };

        match (self.sort_field, self.sort_direction) {
            (SortField::Cost, SortDirection::Descending) => {
                models.sort_by(|a, b| b.cost.total_cmp(&a.cost).then_with(|| tie_breaker(a, b)))
            }
            (SortField::Cost, SortDirection::Ascending) => {
                models.sort_by(|a, b| a.cost.total_cmp(&b.cost).then_with(|| tie_breaker(a, b)))
            }
            (SortField::Tokens, SortDirection::Descending) => models.sort_by(|a, b| {
                b.tokens
                    .total()
                    .cmp(&a.tokens.total())
                    .then_with(|| tie_breaker(a, b))
            }),
            (SortField::Tokens, SortDirection::Ascending) => models.sort_by(|a, b| {
                a.tokens
                    .total()
                    .cmp(&b.tokens.total())
                    .then_with(|| tie_breaker(a, b))
            }),
            (SortField::Date, _) => {
                models.sort_by(|a, b| tie_breaker(a, b));
            }
        }

        models
    }

    pub fn get_sorted_daily(&self) -> Vec<&DailyUsage> {
        let mut daily: Vec<&DailyUsage> = self.data.daily.iter().collect();

        match (self.sort_field, self.sort_direction) {
            (SortField::Cost, SortDirection::Descending) => {
                daily.sort_by(|a, b| b.cost.total_cmp(&a.cost).then_with(|| a.date.cmp(&b.date)))
            }
            (SortField::Cost, SortDirection::Ascending) => {
                daily.sort_by(|a, b| a.cost.total_cmp(&b.cost).then_with(|| a.date.cmp(&b.date)))
            }
            (SortField::Tokens, SortDirection::Descending) => daily.sort_by(|a, b| {
                b.tokens
                    .total()
                    .cmp(&a.tokens.total())
                    .then_with(|| a.date.cmp(&b.date))
            }),
            (SortField::Tokens, SortDirection::Ascending) => daily.sort_by(|a, b| {
                a.tokens
                    .total()
                    .cmp(&b.tokens.total())
                    .then_with(|| a.date.cmp(&b.date))
            }),
            (SortField::Date, SortDirection::Descending) => {
                daily.sort_by(|a, b| b.date.cmp(&a.date))
            }
            (SortField::Date, SortDirection::Ascending) => {
                daily.sort_by(|a, b| a.date.cmp(&b.date))
            }
        }

        daily
    }

    pub fn is_narrow(&self) -> bool {
        self.terminal_width < 80
    }

    pub fn is_very_narrow(&self) -> bool {
        self.terminal_width < 60
    }
}
