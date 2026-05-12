use crate::wsl::Distribution;

#[derive(Debug, Clone)]
pub struct AppState {
    pub distributions: Vec<Distribution>,
    pub selected: usize,
    pub status_line: String,
    pub busy: bool,
    pub pending: Pending,
}

#[derive(Debug, Clone)]
pub enum Pending {
    None,
    ConfirmUnregister { name: String },
    ConfirmShutdown,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            distributions: Vec::new(),
            selected: 0,
            status_line: String::new(),
            busy: false,
            pending: Pending::None,
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
    pub fn selected_distro(&self) -> Option<&Distribution> {
        self.distributions.get(self.selected)
    }
}