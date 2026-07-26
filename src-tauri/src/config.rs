use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ── Config root ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: General,
    #[serde(default)]
    pub providers: Providers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct General {
    /// Global shortcut to summon the main window
    pub hotkey_main: String,
    /// Global shortcut to translate current clipboard selection
    pub hotkey: String,
    /// "light" | "dark" | "auto"
    pub theme: String,
    /// Popup auto-dismiss delay in ms (0 = never)
    pub popup_dismiss_ms: u64,
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

    // MDX-specific
    pub path: Option<String>,

    // DeepL-specific
    pub api_key: Option<String>,

    // DeepL / Google
    pub source_lang: Option<String>,
    pub target_lang: Option<String>,

    // Wikipedia-specific
    pub lang: Option<String>,
}

// ── Defaults ──────────────────────────────────────────────────────────────────

impl Default for Config {
    fn default() -> Self {
        Config {
            general: General {
                hotkey_main: "Alt+W".to_string(),
                hotkey: "Alt+D".to_string(),
                theme: "auto".to_string(),
                popup_dismiss_ms: 4000,
            },
            providers: Providers {
                list: vec![
                    ProviderEntry {
                        kind: "deepl".to_string(),
                        enabled: false,
                        priority: 1,
                        label: Some("DeepL".to_string()),
                        path: None,
                        api_key: None,
                        source_lang: Some("auto".to_string()),
                        target_lang: Some("ZH".to_string()),
                        lang: None,
                    },
                    ProviderEntry {
                        kind: "google".to_string(),
                        enabled: true,
                        priority: 2,
                        label: Some("Google Translate".to_string()),
                        path: None,
                        api_key: None,
                        source_lang: Some("auto".to_string()),
                        target_lang: Some("zh-CN".to_string()),
                        lang: None,
                    },
                    ProviderEntry {
                        kind: "wikipedia".to_string(),
                        enabled: true,
                        priority: 3,
                        label: Some("Wikipedia".to_string()),
                        path: None,
                        api_key: None,
                        source_lang: None,
                        target_lang: None,
                        lang: Some("en".to_string()),
                    },
                ],
            },
        }
    }
}

// ── File I/O ──────────────────────────────────────────────────────────────────

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

    // Guard against empty hotkey strings, which would cause parse_shortcut to
    // return an error ("Shortcut has no key") and crash shortcut registration.
    let defaults = Config::default();
    if cfg.general.hotkey_main.trim().is_empty() {
        log::warn!("hotkey_main is empty in config — falling back to default");
        cfg.general.hotkey_main = defaults.general.hotkey_main;
    }
    if cfg.general.hotkey.trim().is_empty() {
        log::warn!("hotkey (selection) is empty in config — falling back to default");
        cfg.general.hotkey = defaults.general.hotkey;
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
# https://github.com/wispet/wispet
#
# theme: "light" | "dark" | "auto"
# hotkey syntax: modifier+key  e.g. "Alt+D", "Ctrl+Shift+W"

"#;

    let body = toml::to_string_pretty(cfg)
        .context("Failed to serialize config to TOML")?;

    fs::write(&path, format!("{}{}", header, body))
        .with_context(|| format!("Failed to write config to {}", path.display()))?;

    Ok(())
}
