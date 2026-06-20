use crate::app::actions::AppAction;
use crate::app::worker::commands::WorkerCmd;
use crate::app::{AppState, Modal, snapshots};
use crate::config;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, HighlightSpacing};
use ratatui_explorer::{File, FileExplorer, FileExplorerBuilder, Theme};
use ratatui_notifications::{Anchor, Level};

pub fn explorer_theme() -> Theme {
    Theme::default()
        .with_block(Block::default().borders(Borders::ALL).title(" Explorer "))
        .with_style(Style::default().fg(Color::Gray))
        .with_dir_style(Style::default().fg(Color::Cyan))
        .with_item_style(Style::default().fg(Color::White))
        .with_highlight_item_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .with_highlight_dir_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
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
        AppAction::Quit => {
            state.should_quit = true;
            vec![]
        }
        AppAction::Help => {
            state.modal = Modal::Help;
            vec![]
        }
        AppAction::MoveSelection(delta) => {
            state.move_selection(delta);
            vec![]
        }
        AppAction::RunSelected => {
            let names: Vec<String> = state.targeted_distros();

            if names.is_empty() {
                return vec![];
            }
            state.clear_multi_select();
            let cmds = names
                .into_iter()
                .map(WorkerCmd::RunDistro)
                .collect::<Vec<_>>();
            vec![WorkerCmd::Batch(cmds)]
        }
        AppAction::OpenShell => {
            let names: Vec<String> = state.targeted_distros();
            if names.is_empty() {
                return vec![];
            }
            state.clear_multi_select();
            let cmds = names
                .into_iter()
                .map(WorkerCmd::OpenShell)
                .collect::<Vec<_>>();
            vec![WorkerCmd::Batch(cmds)]
        }
        AppAction::Terminate => {
            let names = state.targeted_distros();
            if names.is_empty() {
                return vec![];
            }
            state.clear_multi_select();
            let cmds = names
                .into_iter()
                .map(WorkerCmd::Terminate)
                .collect::<Vec<_>>();
            vec![WorkerCmd::Batch(cmds)]
        }
        AppAction::SetDefault => {
            let name = state.selected_distro().map(|d| d.name.clone());

            name.map(WorkerCmd::SetDefault).into_iter().collect()
        }
        AppAction::UnregisterPrompt => {
            let names: Vec<String> = state.targeted_distros();

            if !names.is_empty() {
                state.modal = Modal::ConfirmUnregister { names }
            }

            vec![]
        }
        AppAction::ShutdownPrompt => {
            state.modal = Modal::ConfirmShutdown;

            vec![]
        }
        AppAction::ExportPrompt => {
            let names: Vec<String> = state.targeted_distros();

            if !names.is_empty() {
                let mut explorer = new_explorer();
                let _ = explorer.set_filter_map(only_dirs);
                state.modal = Modal::ExportPicker {
                    distros: names,
                    explorer,
                };
            }

            vec![]
        }
        AppAction::ImportPrompt => {
            let mut explorer = new_explorer();
            let _ = explorer.set_filter_map(tar_or_dirs);
            state.modal = Modal::ImportTarPicker { explorer };
            vec![]
        }
        AppAction::CustomActionsPrompt => {
            if let Some(distro) = state.selected_distro().map(|d| d.name.clone()) {
                let actions = config::load_or_create().custom_actions;
                if actions.is_empty() {
                    state.notify(
                        "No custom actions configured in settings.json.".to_string(),
                        Level::Info,
                        Anchor::TopRight,
                        2,
                    );
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
            state.notify(
                "Cancelled!".to_string(),
                Level::Info,
                ratatui_notifications::Anchor::TopCenter,
                2,
            );
            vec![]
        }
        AppAction::TogglePin => {
            let names: Vec<String> = state.targeted_distros();

            for name in &names {
                state.toggle_pin(name);
            }

            state.clear_multi_select();

            state.clamp_selection();
            if names.len() == 1 {
                let d = &names[0];
                let msg = if state.pinned.contains(d) {
                    format!("Pinned {}", d)
                } else {
                    format!("Unpinned {}", d)
                };
                state.notify(msg, Level::Info, Anchor::TopRight, 2);
            } else {
                state.notify(
                    format!("Toggled pin for {} distros", names.len()),
                    Level::Info,
                    Anchor::TopRight,
                    2,
                );
            }
            vec![]
        }
        AppAction::ToggleMultiSelect => {
            if let Some(name) = state.selected_distro().map(|d| d.name.clone()) {
                state.toggle_multi_select(&name);
                let msg = if state.selected_multi.contains(&name) {
                    format!("Marked: {}", name)
                } else {
                    format!("Unmarked: {}", name)
                };
                state.notify(msg, Level::Info, Anchor::TopRight, 2);
            }
            vec![]
        }
        AppAction::ClonePrompt => {
            if let Some(distro) = state.selected_distro().map(|d| d.name.clone()) {
                state.modal = Modal::CloneDistro {
                    distro,
                    new_name: String::new(),
                }
            };
            vec![]
        }
        AppAction::RollBackPrompt => {
            let distros = snapshots::list_snapshot_distros();
            if distros.is_empty() {
                state.notify(
                    "No Snapshots found".to_string(),
                    Level::Info,
                    Anchor::TopRight,
                    2,
                );
            } else {
                state.modal = Modal::RollBackDistroPicker {
                    distros,
                    selected: 0,
                };
            }

            vec![]
        }
        AppAction::SnapshotPrompt => {
            let Some(distro) = state.selected_distro().map(|d| d.name.clone()) else {
                state.notify(
                    "No distro selected".to_string(),
                    Level::Info,
                    Anchor::TopRight,
                    2,
                );
                return vec![];
            };

            match snapshots::next_snapshot_path(&distro) {
                Ok(output) => {
                    state.notify(
                        format!("Snapshotting `{distro}`..."),
                        Level::Info,
                        Anchor::TopRight,
                        2,
                    );
                    vec![WorkerCmd::Export { distro, output }]
                }
                Err(e) => {
                    state.notify(
                        format!("Snapshot path error: {e}"),
                        Level::Error,
                        Anchor::TopRight,
                        2,
                    );
                    vec![]
                }
            }
        }
        AppAction::SnapshotManagerPrompt => {
            let distros = snapshots::list_snapshot_distros();
            if distros.is_empty() {
                state.notify(
                    "No snapshots yet! - press z to create one".to_string(),
                    Level::Info,
                    Anchor::TopRight,
                    2,
                );
                return vec![];
            }
            let distro_idx = 0;
            let snapshots = snapshots::list_snapshot_infos(&distros[distro_idx]);
            state.modal = Modal::SnapshotManager {
                distros,
                distro_idx,
                snapshots,
                snap_idx: 0,
                focus_right: false,
            };
            vec![]
        }
        AppAction::Ignore => vec![],
    }
}
