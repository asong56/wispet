use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: General,
    #[serde(default)]
    pub providers: Providers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct General {
    /// Hotkey that opens the search box.
    pub hotkey_main: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Providers {
    pub list: Vec<ProviderEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEntry {
    /// "mdx" | "deepl" | "google" | "wikipedia"
    #[serde(rename = "type")]
    pub kind: String,
    pub enabled: bool,
    pub priority: u32,
    pub label: Option<String>,
    pub path: Option<String>,
    pub api_key: Option<String>,
    pub source_lang: Option<String>,
    pub target_lang: Option<String>,
    pub lang: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: General {
                hotkey_main: "Ctrl+Alt+Space".to_string(),
            },
            providers: Providers {
                list: vec![ProviderEntry {
                    kind: "mdx".to_string(),
                    enabled: true,
                    priority: 1,
                    label: Some("Vocabulary.com".to_string()),
                    // Placeholder — edit this path in wispet.toml to point
                    // at your actual vocabulary.com.mdx file.
                    path: Some("vocabulary.com.mdx".to_string()),
                    api_key: None,
                    source_lang: None,
                    target_lang: None,
                    lang: None,
                }],
            },
        }
    }
}

pub fn config_path() -> PathBuf {
    let base = config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("wispet").join("config.toml")
}

pub fn load_or_create() -> Result<Config> {
    let path = config_path();

    if !path.exists() {
        let cfg = Config::default();
        save(&cfg)?;
        return Ok(cfg);
    }

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config at {}", path.display()))?;

    let mut cfg: Config = toml::from_str(&raw)
        .with_context(|| "Failed to parse config.toml — check for syntax errors")?;

    // An empty hotkey would make parse_shortcut error and crash registration.
    let defaults = Config::default();
    if cfg.general.hotkey_main.trim().is_empty() {
        log::warn!("hotkey_main is empty in config — falling back to default");
        cfg.general.hotkey_main = defaults.general.hotkey_main;
    }

    Ok(cfg)
}

pub fn save(cfg: &Config) -> Result<()> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory at {}", parent.display()))?;
    }

    let header = r#"# Wispet configuration
#
# This is the ONLY place Wispet is configured — there is no in-app
# settings UI. Edit this file, then restart Wispet for changes to take
# effect (config is read once at startup).
#
# hotkey_main : opens the search box
#               hotkey syntax: modifier+key, e.g. "Ctrl+Alt+Space", "Ctrl+Shift+W"
#
# [[providers.list]] entries are tried in `priority` order (lowest first).
# type = "mdx"       — a local .mdx dictionary file (path required)
# type = "deepl"     — DeepL API (api_key required)
# type = "google"    — Google Translate (no key required)
# type = "wikipedia" — Wikipedia summary (no key required)

"#;

    let body = toml::to_string_pretty(cfg)
        .context("Failed to serialize config to TOML")?;

    fs::write(&path, format!("{}{}", header, body))
        .with_context(|| format!("Failed to write config to {}", path.display()))?;

    Ok(())
}
