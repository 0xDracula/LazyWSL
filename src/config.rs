use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomActions {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeoutConfig {
    pub quick_secs: u64,
    pub default_secs: u64,
    pub long_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub timeouts: TimeoutConfig,
    pub refresh_secs: u64,
    #[serde(default)]
    pub custom_actions: Vec<CustomActions>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            timeouts: TimeoutConfig {
                quick_secs: 5,
                default_secs: 15,
                long_secs: 60,
            },
            refresh_secs: 2,
            custom_actions: Vec::new(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    let proj = ProjectDirs::from("com", "lazywsl", "LazyWSL")
        .expect("Couldn't find config dir");

    proj.config_dir().to_path_buf()
}

pub fn config_path() -> PathBuf {
    config_dir().join("settings.json")
}

pub fn load_or_create() -> AppConfig {
    let path = config_path();

    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&data) {
                return cfg;
            }
        }
    }

    let cfg = AppConfig::default();
    let _ = save(&cfg);
    cfg
}

pub fn save(cfg: &AppConfig) -> std::io::Result<()> {
    fs::create_dir_all(config_dir())?;
    let content = serde_json::to_string_pretty(cfg)?;
    fs::write(config_path(), content)
}