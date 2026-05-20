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

pub fn reduce(state: &mut AppState, action: AppAction) -> Vec<WorkerCmd> {
    match action {
        AppAction::Quit => { state.should_quit = true; vec![] }
        AppAction::Help => { state.modal = Modal::Help; vec![] }
        AppAction::MoveSelection(delta) => { state.move_selection(delta); vec![] }
        AppAction::RunSelected => {
            let names: Vec<String> = state.targeted_distros();

            if names.is_empty() {
                return vec![]
            }
            state.clear_multi_select();
            let cmds = names.into_iter().map(WorkerCmd::RunDistro).collect::<Vec<_>>();
            vec![WorkerCmd::Batch(cmds)]
        },
        AppAction::OpenShell => {
            let names: Vec<String> = state.targeted_distros();
            if names.is_empty() {
                return vec![]
            }
            state.clear_multi_select();
            let cmds = names.into_iter().map(WorkerCmd::OpenShell).collect::<Vec<_>>();
            vec![WorkerCmd::Batch(cmds)]
        }
        AppAction::Terminate => {
            let names = state.targeted_distros();
            if names.is_empty() {
                return vec![]
            }
            state.clear_multi_select();
            let cmds = names.into_iter().map(WorkerCmd::Terminate).collect::<Vec<_>>();
            vec![WorkerCmd::Batch(cmds)]
        },
        AppAction::SetDefault => {
            let name = state.selected_distro().map(|d| d.name.clone());

            name.map(|n| WorkerCmd::SetDefault(n)).into_iter().collect()
        },
        AppAction::UnregisterPrompt => {
            let names: Vec<String> = state.targeted_distros();

            if !names.is_empty() {
                state.modal = Modal::ConfirmUnregister { names }
            }

            vec![]
        }
        AppAction::ShutdownPrompt => {
            state.modal = Modal::ConfirmShutdown;
            state.status_line = "Shutdown stops all WSL2 VMs, press y to confirm!".to_string();

            vec![]
        }
        AppAction::ExportPrompt => {
            let names: Vec<String> = state.targeted_distros();

            if !names.is_empty() {
                let mut explorer = new_explorer();
                let _ = explorer.set_filter_map(only_dirs);
                state.modal = Modal::ExportPicker { distros: names, explorer };
            }

            vec![]
        }
        AppAction::ImportPrompt => {
            let mut explorer = new_explorer();
            let _ = explorer.set_filter_map(tar_or_dirs);
            state.modal = Modal::ImportTarPicker {
                explorer
            };
            vec![]
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
            vec![]
        }
        AppAction::SearchPrompt => {
            state.search_active = true;
            vec![]
        }
        AppAction::ClearSearch => {
            state.search_query.clear();
            state.selected = 0;
            state.clamp_selection();
            state.search_active = false;
            state.status_line = "Search Cleared".to_string();
            vec![]
        }
        AppAction::TogglePin => {
            let names: Vec<String> = state.targeted_distros();

            for name in &names {
                state.toggle_pin(&name);
            }

            state.clear_multi_select();

            state.clamp_selection();
            if names.len() == 1 {
                let d = &names[0];
                state.status_line = if state.pinned.contains(d) {
                    format!("Pinned {}", d)
                } else {
                    format!("Unpinned {}", d)
                };
            } else {
                state.status_line = format!("Toggled pin for {} distros", names.len());
            }
            vec![]
        }
        AppAction::ToggleMultiSelect => {
            if let Some(name) = state.selected_distro().map(|d| d.name.clone()) {
                state.toggle_multi_select(&name);
                state.status_line = if state.selected_multi.contains(&name) {
                    format!("Marked: {}", name)
                } else {
                    format!("Unmarked: {}", name)
                };
            }
            vec![]
        }
        AppAction::Ignore => vec![],
    }
}