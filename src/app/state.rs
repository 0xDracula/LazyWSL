use std::path::PathBuf;
use ratatui_explorer::FileExplorer;
use crate::config::CustomActions;
use crate::wsl::Distribution;

pub struct AppState {
    pub distributions: Vec<Distribution>,
    pub selected: usize,
    pub status_line: String,
    pub busy: bool,
    pub modal: Modal,
    pub should_quit: bool,
}

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
    ActionOuptut { distro: String, action_name: String, lines: Vec<String>, finished: bool },
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
        }
    }
}

impl AppState {
    pub fn clamp_selection(&mut self) {
        if self.distributions.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.distributions.len() - 1);
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.distributions.is_empty() {
            return;
        }

        let len = self.distributions.len();
        let i = self.selected as isize + delta;
        let i = i.clamp(0, (len - 1) as isize) as usize;
        self.selected = i;
    }

    pub fn selected_distro(&self) -> Option<&Distribution> {
        self.distributions.get(self.selected)
    }
}