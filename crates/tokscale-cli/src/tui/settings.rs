use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::themes::ThemeName;

const DEFAULT_SUBPROCESS_TIMEOUT_MS: u64 = 300_000; // 5 minutes
const MIN_SUBPROCESS_TIMEOUT_MS: u64 = 5_000; // 5 seconds
const MAX_SUBPROCESS_TIMEOUT_MS: u64 = 3_600_000; // 1 hour

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    pub auto_refresh_interval: u64,
    pub enabled_sources: Vec<String>,
    #[serde(default = "default_subprocess_timeout")]
    pub subprocess_timeout_ms: u64,
}

fn default_subprocess_timeout() -> u64 {
    DEFAULT_SUBPROCESS_TIMEOUT_MS
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "green".to_string(),
            auto_refresh_interval: 0,
            enabled_sources: vec![
                "OpenCode".to_string(),
                "Claude".to_string(),
                "Codex".to_string(),
                "Cursor".to_string(),
                "Gemini".to_string(),
                "Amp".to_string(),
                "Droid".to_string(),
                "OpenClaw".to_string(),
            ],
            subprocess_timeout_ms: DEFAULT_SUBPROCESS_TIMEOUT_MS,
        }
    }
}

impl Settings {
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("tokscale");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        Ok(config_dir.join("settings.json"))
    }

    pub fn load() -> Self {
        Self::config_path()
            .ok()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn theme_name(&self) -> ThemeName {
        self.theme.parse().unwrap_or(ThemeName::Green)
    }

    pub fn set_theme(&mut self, theme: ThemeName) {
        self.theme = theme.as_str().to_string();
    }

    /// Get subprocess timeout as Duration, with env var override.
    /// Priority: TOKSCALE_SUBPROCESS_TIMEOUT_MS env var > settings.json > default (5 min)
    pub fn get_subprocess_timeout(&self) -> Duration {
        let timeout_ms = if let Ok(env_val) = std::env::var("TOKSCALE_SUBPROCESS_TIMEOUT_MS") {
            env_val.parse::<u64>().unwrap_or(self.subprocess_timeout_ms)
        } else {
            self.subprocess_timeout_ms
        };

        let clamped = timeout_ms.clamp(MIN_SUBPROCESS_TIMEOUT_MS, MAX_SUBPROCESS_TIMEOUT_MS);
        Duration::from_millis(clamped)
    }
}
