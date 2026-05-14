use ratatree::FilePickerState;
use crate::app::actions::AppAction;
use crate::app::{AppState, Modal};
use crate::app::worker::commands::WorkerCmd;

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
                state.modal = Modal::ExportPicker {
                    distro: d.name.clone(),
                    picker: FilePickerState::builder()
                        .start_dir(std::env::current_dir().unwrap())
                        .mode(ratatree::PickerMode::DirsOnly)
                        .build(),
                };
            }
            None
        }
        AppAction::ImportPrompt => {
            state.modal = Modal::ImportTarPicker {
                picker: FilePickerState::builder()
                    .start_dir(std::env::current_dir().unwrap())
                    .build(),
            };
            None
        }
        AppAction::Ignore => None,
    }
}