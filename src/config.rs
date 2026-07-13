use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomActions {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeymapConfig {
    #[serde(default = "default_quit_keys")]
    pub quit: Vec<String>,
    #[serde(default = "default_help_keys")]
    pub help: Vec<String>,
    #[serde(default = "default_health_keys")]
    pub health: Vec<String>,
    #[serde(default = "default_run_keys")]
    pub run: Vec<String>,
    #[serde(default = "default_open_shell_keys")]
    pub open_shell: Vec<String>,
    #[serde(default = "default_terminate_keys")]
    pub terminate: Vec<String>,
    #[serde(default = "defailt_set_default_keys")]
    pub set_default: Vec<String>,
    #[serde(default = "default_unregister_keys")]
    pub unregister: Vec<String>,
    #[serde(default = "default_shutdown_keys")]
    pub shutdown: Vec<String>,
    #[serde(default = "default_export_keys")]
    pub export: Vec<String>,
    #[serde(default = "default_import_keys")]
    pub import: Vec<String>,
    #[serde(default = "default_custom_actions_keys")]
    pub custom_actions: Vec<String>,
    #[serde(default = "default_search_keys")]
    pub search: Vec<String>,
    #[serde(default = "default_clear_search_keys")]
    pub clear_search: Vec<String>,
    #[serde(default = "default_toggle_pin_keys")]
    pub toggle_pin: Vec<String>,
    #[serde(default = "default_toggle_multi_select_keys")]
    pub toggle_multi_select: Vec<String>,
    #[serde(default = "default_move_down_keys")]
    pub move_down: Vec<String>,
    #[serde(default = "default_move_up_keys")]
    pub move_up: Vec<String>,
    #[serde(default = "default_clone_keys")]
    pub clone: Vec<String>,
    #[serde(default = "default_rollback_keys")]
    pub rollback: Vec<String>,
    #[serde(default = "default_snapshot_keys")]
    pub snapshot: Vec<String>,
    #[serde(default = "default_snapshot_manager_keys")]
    pub snapshot_manager: Vec<String>,
    #[serde(default = "default_catalog_keys")]
    pub catalog: Vec<String>,
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
    pub keymaps: KeymapConfig,
    #[serde(default)]
    pub custom_actions: Vec<CustomActions>,
}

fn keys(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn default_quit_keys() -> Vec<String> {
    keys(&["q", "Q", "esc"])
}

fn default_help_keys() -> Vec<String> {
    keys(&["?", "h"])
}

fn default_health_keys() -> Vec<String> {
    keys(&["H"])
}

fn default_run_keys() -> Vec<String> {
    keys(&["r", "R"])
}

fn default_open_shell_keys() -> Vec<String> {
    keys(&["enter"])
}

fn default_terminate_keys() -> Vec<String> {
    keys(&["t", "T"])
}

fn defailt_set_default_keys() -> Vec<String> {
    keys(&["d", "D"])
}

fn default_unregister_keys() -> Vec<String> {
    keys(&["u", "U"])
}

fn default_shutdown_keys() -> Vec<String> {
    keys(&["s"])
}

fn default_export_keys() -> Vec<String> {
    keys(&["e", "E"])
}

fn default_import_keys() -> Vec<String> {
    keys(&["i", "I"])
}

fn default_custom_actions_keys() -> Vec<String> {
    keys(&["a", "A"])
}

fn default_search_keys() -> Vec<String> {
    keys(&["/"])
}

fn default_clear_search_keys() -> Vec<String> {
    keys(&["c", "C"])
}

fn default_toggle_pin_keys() -> Vec<String> {
    keys(&["p", "P"])
}

fn default_toggle_multi_select_keys() -> Vec<String> {
    keys(&["space"])
}

fn default_move_down_keys() -> Vec<String> {
    keys(&["down"])
}

fn default_move_up_keys() -> Vec<String> {
    keys(&["up"])
}

fn default_clone_keys() -> Vec<String> {
    keys(&["n", "N"])
}

fn default_rollback_keys() -> Vec<String> {
    keys(&["b", "B"])
}

fn default_snapshot_keys() -> Vec<String> {
    keys(&["z", "Z"])
}

fn default_snapshot_manager_keys() -> Vec<String> {
    keys(&["S"])
}

fn default_catalog_keys() -> Vec<String> {
    keys(&["o"])
}

impl Default for KeymapConfig {
    fn default() -> Self {
        Self {
            quit: default_quit_keys(),
            help: default_help_keys(),
            health: default_health_keys(),
            run: default_run_keys(),
            open_shell: default_open_shell_keys(),
            terminate: default_terminate_keys(),
            set_default: defailt_set_default_keys(),
            unregister: default_unregister_keys(),
            shutdown: default_shutdown_keys(),
            export: default_export_keys(),
            import: default_import_keys(),
            custom_actions: default_custom_actions_keys(),
            search: default_search_keys(),
            clear_search: default_clear_search_keys(),
            toggle_pin: default_toggle_pin_keys(),
            toggle_multi_select: default_toggle_multi_select_keys(),
            move_down: default_move_down_keys(),
            move_up: default_move_up_keys(),
            clone: default_clone_keys(),
            rollback: default_rollback_keys(),
            snapshot: default_snapshot_keys(),
            snapshot_manager: default_snapshot_manager_keys(),
            catalog: default_catalog_keys(),
        }
    }
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
            keymaps: KeymapConfig::default(),
            custom_actions: Vec::new(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    let proj = ProjectDirs::from("com", "lazywsl", "LazyWSL").expect("Couldn't find config dir");

    proj.config_dir().to_path_buf()
}

pub fn config_path() -> PathBuf {
    config_dir().join("settings.json")
}

pub fn load_or_create() -> AppConfig {
    let path = config_path();

    if path.exists()
        && let Ok(data) = fs::read_to_string(&path)
        && let Ok(cfg) = serde_json::from_str(&data)
    {
        return cfg;
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
