use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use ratatui_explorer::FileExplorer;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use crate::config::CustomActions;
use crate::wsl::Distribution;

pub struct AppState {
    pub distributions: Vec<Distribution>,
    pub selected: usize,
    pub status_line: String,
    pub busy: bool,
    pub modal: Modal,
    pub should_quit: bool,
    pub search_query: String,
    pub search_active: bool,
    pub pinned: HashSet<String>,
}

#[derive(Serialize, Deserialize)]
struct PinsSer { pins: Vec<String> }

pub enum Modal {
    None,
    Help,
    ConfirmUnregister { name: String },
    ConfirmShutdown,
    ExportPicker { distro: String, explorer: FileExplorer },
    ImportTarPicker { explorer: FileExplorer },
    ImportInstallPicker { tar_path: PathBuf, explorer: FileExplorer },
    ImportNameInput { tar_path: PathBuf, install_dir: PathBuf, input: String },
    CustomActionsMenu { distro: String, actions: Vec<CustomActions>, selected: usize },
    ActionOuptut { distro: String, action_name: String, output: String, finished: bool, input: String, input_tx: Sender<String> },
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            distributions: Vec::new(),
            selected: 0,
            status_line: String::new(),
            busy: false,
            modal: Modal::None,
            should_quit: false,
            search_active: false,
            search_query: String::new(),
            pinned: crate::app::state::load_pins(),
        }
    }
}

impl AppState {
    pub fn clamp_selection(&mut self) {
        let len = self.filtered_indices().len();

        if len == 0 {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(len - 1);
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.filtered_indices().len();

        if len == 0 {
            return;
        }

        let i = self.selected as isize + delta;
        let i = i.clamp(0, (len - 1) as isize) as usize;
        self.selected = i;
    }

    pub fn selected_distro(&self) -> Option<&Distribution> {
        let indices = self.filtered_indices();
        let idx = indices.get(self.selected)?;
        self.distributions.get(*idx)
    }

    pub fn filtered_indices(&self) -> Vec<usize> {
        let query = self.search_query.trim().to_lowercase();
        let mut pins: Vec<usize> = Vec::new();
        let mut rest: Vec<usize> = Vec::new();
        for (i, d) in self.distributions.iter().enumerate() {
            if !query.is_empty() && !d.name.to_lowercase().contains(&query) {
                continue;
            }
            if self.pinned.contains(&d.name) {
                pins.push(i);
            } else {
                rest.push(i);
            }
        }

        pins.extend(rest);
        pins
    }

    pub fn toggle_pin(&mut self, name: &str) {
        if self.pinned.contains(name) {
            self.pinned.remove(name);
        } else {
            self.pinned.insert(name.to_string());
        }
        save_pins(&self.pinned);
    }
}

fn pins_path() -> PathBuf {
    let mut dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("LazyWSL");
    fs::create_dir_all(&dir).ok();
    dir.push("pins.json");
    dir
}

pub fn load_pins() -> HashSet<String> {
    let path = pins_path();
    if let Ok(s) = fs::read_to_string(&path) {
        if let Ok(data) = serde_json::from_str::<PinsSer>(&s) {
            return data.pins.into_iter().collect();
        }
    }
    HashSet::new()
}

pub fn save_pins(pins: &HashSet<String>) {
    let path = pins_path();
    let data = PinsSer { pins: pins.iter().cloned().collect() };
    if let Ok(s) = serde_json::to_string_pretty(&data) {
        let _ = fs::write(path, s);
    }
}