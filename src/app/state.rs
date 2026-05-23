use crate::config::CustomActions;
use crate::wsl::Distribution;
use ratatui_explorer::FileExplorer;
use ratatui_notifications::{AutoDismiss, Notifications, Timing};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

pub struct AppState {
    pub distributions: Vec<Distribution>,
    pub selected: usize,
    pub notifications: ratatui_notifications::Notifications,
    pub busy: bool,
    pub modal: Modal,
    pub should_quit: bool,
    pub search_query: String,
    pub search_active: bool,
    pub pinned: HashSet<String>,
    pub selected_multi: HashSet<String>,
}

#[derive(Serialize, Deserialize)]
struct PinsSer {
    pins: Vec<String>,
}

pub enum Modal {
    None,
    Help,
    ConfirmUnregister {
        names: Vec<String>,
    },
    ConfirmShutdown,
    ExportPicker {
        distros: Vec<String>,
        explorer: FileExplorer,
    },
    ImportTarPicker {
        explorer: FileExplorer,
    },
    ImportInstallPicker {
        tar_path: PathBuf,
        explorer: FileExplorer,
    },
    ImportNameInput {
        tar_path: PathBuf,
        install_dir: PathBuf,
        input: String,
    },
    CustomActionsMenu {
        distro: String,
        actions: Vec<CustomActions>,
        selected: usize,
    },
    ActionOutput {
        distro: String,
        action_name: String,
        output: String,
        finished: bool,
        input: String,
        input_tx: Sender<String>,
    },
}

impl Default for AppState {
    fn default() -> Self {
        let notifications = Notifications::new()
            .max_concurrent(Some(5))
            .overflow(ratatui_notifications::Overflow::DiscardOldest);

        Self {
            distributions: Vec::new(),
            selected: 0,
            notifications,
            busy: false,
            modal: Modal::None,
            should_quit: false,
            search_active: false,
            search_query: String::new(),
            pinned: crate::app::state::load_pins(),
            selected_multi: HashSet::new(),
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

    // TODO! fix slow animations
    pub fn notify(
        &mut self,
        msg: String,
        level: ratatui_notifications::Level,
        anchor: ratatui_notifications::Anchor,
        dismiss: u64,
    ) {
        let notif = ratatui_notifications::Notification::new(msg)
            .level(level)
            .anchor(anchor)
            .auto_dismiss(AutoDismiss::After(Duration::from_secs(dismiss)))
            .timing(
                Timing::Fixed(Duration::from_millis(200)),
                Timing::Fixed(Duration::from_secs(2)),
                Timing::Fixed(Duration::from_millis(300)),
            )
            .build()
            .unwrap();

        let _ = self.notifications.add(notif);
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

    pub fn targeted_distros(&self) -> Vec<String> {
        if !self.selected_multi.is_empty() {
            self.selected_multi.iter().cloned().collect()
        } else {
            self.selected_distro()
                .map_or(vec![], |d| vec![d.name.clone()])
        }
    }

    pub fn toggle_pin(&mut self, name: &str) {
        if self.pinned.contains(name) {
            self.pinned.remove(name);
        } else {
            self.pinned.insert(name.to_string());
        }
        save_pins(&self.pinned);
    }

    pub fn toggle_multi_select(&mut self, name: &str) {
        if self.selected_multi.contains(name) {
            self.selected_multi.remove(name);
        } else {
            self.selected_multi.insert(name.to_owned());
        }
    }

    pub fn clear_multi_select(&mut self) {
        self.selected_multi.clear();
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
    if let Ok(s) = fs::read_to_string(&path)
        && let Ok(data) = serde_json::from_str::<PinsSer>(&s)
    {
        return data.pins.into_iter().collect();
    }
    HashSet::new()
}

pub fn save_pins(pins: &HashSet<String>) {
    let path = pins_path();
    let data = PinsSer {
        pins: pins.iter().cloned().collect(),
    };
    if let Ok(s) = serde_json::to_string_pretty(&data) {
        let _ = fs::write(path, s);
    }
}
