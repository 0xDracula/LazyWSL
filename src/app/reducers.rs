use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, HighlightSpacing};
use ratatui_explorer::{FileExplorer, File, Theme, FileExplorerBuilder};
use crate::app::actions::AppAction;
use crate::app::{AppState, Modal};
use crate::app::worker::commands::WorkerCmd;
use crate::config;

pub fn explorer_theme() -> Theme {
    Theme::default()
        .with_block(Block::default().borders(Borders::ALL).title(" Explorer "))
        .with_style(Style::default().fg(Color::Gray))
        .with_dir_style(Style::default().fg(Color::Cyan))
        .with_item_style(Style::default().fg(Color::White))
        .with_highlight_item_style(Style::default().fg(Color::Black).bg(Color::Magenta).add_modifier(Modifier::BOLD))
        .with_highlight_dir_style(Style::default().fg(Color::Black).bg(Color::Blue).add_modifier(Modifier::BOLD))
        .with_scroll_padding(2)
        .with_highlight_symbol("▶ ")
        .with_highlight_spacing(HighlightSpacing::Always)
        .with_title_bottom(|_| ratatui::text::Line::from(" ↑↓←→ to move · Enter to select "))
        .with_title_top(|f| ratatui::text::Line::from(format!(" {}", f.cwd().display())))
}

fn new_explorer() -> FileExplorer {
    FileExplorerBuilder::build_with_theme(explorer_theme()).unwrap()
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
        AppAction::CustomActionsPrompt => {
            if let Some(distro) = state.selected_distro().map(|d| d.name.clone()) {
                let actions = config::load_or_create().custom_actions;
                if actions.is_empty() {
                    state.status_line = "No custom actions configured in settings.json.".to_string();
                } else {
                    state.modal = Modal::CustomActionsMenu {
                        distro,
                        actions,
                        selected: 0,
                    }
                }
            }
            None
        }
        AppAction::Ignore => None,
    }
}