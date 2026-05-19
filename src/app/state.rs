use std::path::PathBuf;
use ratatui_explorer::FileExplorer;
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
        self.distributions
            .iter()
            .enumerate()
            .filter(|(_, d)| {
                query.is_empty() | d.name.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect()
    }
}