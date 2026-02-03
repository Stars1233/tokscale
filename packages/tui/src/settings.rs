use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::themes::ThemeName;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    pub auto_refresh_interval: u64,
    pub enabled_sources: Vec<String>,
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
}
