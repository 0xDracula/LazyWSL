use ratatui_explorer::{FileExplorer, File, Theme, FileExplorerBuilder};
use crate::app::actions::AppAction;
use crate::app::{AppState, Modal};
use crate::app::worker::commands::WorkerCmd;

fn new_explorer() -> FileExplorer {
    let theme = Theme::default().add_default_title();
    FileExplorerBuilder::build_with_theme(theme).unwrap()
}

fn only_dirs(f: File) -> Option<File> {
    if f.is_dir { Some(f) } else { None }
}

fn is_tar_name(name: &str) -> bool {
    name.ends_with(".tar") || name.ends_with(".tar.gz") || name.ends_with(".tgz")
}

fn tar_or_dirs(f: File) -> Option<File> {
    if f.is_dir || is_tar_name(&f.name) {
        Some(f)
    } else {
        None
    }
}

pub fn reduce(state: &mut AppState, action: AppAction) -> Option<WorkerCmd> {
    match action {
        AppAction::Quit => { state.should_quit = true; None }
        AppAction::Help => { state.modal = Modal::Help; None }
        AppAction::MoveSelection(delta) => { state.move_selection(delta); None }
        AppAction::RunSelected => state.selected_distro().map(|d| WorkerCmd::RunDistro(d.name.clone())),
        AppAction::OpenShell => state.selected_distro().map(|d| WorkerCmd::OpenShell(d.name.clone())),
        AppAction::Terminate => state.selected_distro().map(|d| WorkerCmd::Terminate(d.name.clone())),
        AppAction::SetDefault => state.selected_distro().map(|d| WorkerCmd::SetDefault(d.name.clone())),
        AppAction::UnregisterPrompt => {
            if let Some(d) = state.selected_distro() {
                state.modal = Modal::ConfirmUnregister { name: d.name.clone() };
            }
            None
        }
        AppAction::ShutdownPrompt => {
            state.modal = Modal::ConfirmShutdown;
            state.status_line = "Shutdown stops all WSL2 VMs, press y to confirm!".to_string();

            None
        }
        AppAction::ExportPrompt => {
            if let Some(d) = state.selected_distro() {
                let mut explorer = new_explorer();
                let _ = explorer.set_filter_map(only_dirs);
                state.modal = Modal::ExportPicker {
                    distro: d.name.clone(),
                    explorer,
                };
            }
            None
        }
        AppAction::ImportPrompt => {
            let mut explorer = new_explorer();
            let _ = explorer.set_filter_map(tar_or_dirs);
            state.modal = Modal::ImportTarPicker {
                explorer
            };
            None
        }
        AppAction::Ignore => None,
    }
}