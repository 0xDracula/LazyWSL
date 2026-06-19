use crate::app::controller::dispatch;
use crate::app::reducers::explorer_theme;
use crate::app::worker::commands::WorkerCmd;
use crate::app::{AppState, Modal, snapshots};
use crate::config;
use crate::ui::Component;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui_explorer::FileExplorerBuilder;
use ratatui_notifications::{Anchor, Level};
use std::fs;
use std::future::Future;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

pub struct ModalComponent;

impl ModalComponent {
    pub fn new() -> Self {
        ModalComponent
    }

    async fn handle_event_inner(
        &mut self,
        state: &mut AppState,
        cmd_tx: &mpsc::Sender<WorkerCmd>,
        ev: Event,
    ) -> ControlFlow<()> {
        if matches!(state.modal, Modal::None) {
            return ControlFlow::Continue(());
        }

        match ev {
            Event::Key(key) => Self::handle_modal_key(state, cmd_tx, key.code).await,
            _ => ControlFlow::Continue(()),
        }
    }

    async fn handle_modal_key(
        state: &mut AppState,
        cmd_tx: &mpsc::Sender<WorkerCmd>,
        code: KeyCode,
    ) -> ControlFlow<()> {
        let modal = std::mem::replace(&mut state.modal, Modal::None);
        match modal {
            Modal::Help => {
                if matches!(code, KeyCode::Esc) {
                    state.modal = Modal::None;
                } else {
                    state.modal = Modal::Help;
                }
                ControlFlow::Continue(())
            }

            Modal::ConfirmUnregister { names } => match code {
                KeyCode::Char('y') => {
                    state.clear_multi_select();
                    let cmds = names
                        .into_iter()
                        .map(WorkerCmd::Unregister)
                        .collect::<Vec<_>>();
                    dispatch(state, cmd_tx, WorkerCmd::Batch(cmds)).await;
                    ControlFlow::Continue(())
                }
                KeyCode::Char('n') => {
                    state.notify("Cancelled".to_string(), Level::Info, Anchor::TopRight, 2);
                    ControlFlow::Continue(())
                }
                KeyCode::Char('q') => ControlFlow::Break(()),
                _ => {
                    state.modal = Modal::ConfirmUnregister { names };
                    ControlFlow::Continue(())
                }
            },

            Modal::ConfirmShutdown => match code {
                KeyCode::Char('y') => {
                    dispatch(state, cmd_tx, WorkerCmd::Shutdown).await;
                    ControlFlow::Continue(())
                }
                KeyCode::Char('n') => {
                    state.notify("Cancelled".to_string(), Level::Info, Anchor::TopRight, 2);
                    ControlFlow::Continue(())
                }
                KeyCode::Char('q') => ControlFlow::Break(()),
                _ => {
                    state.modal = Modal::ConfirmShutdown;
                    ControlFlow::Continue(())
                }
            },

            Modal::ImportNameInput {
                tar_path,
                install_dir,
                mut input,
            } => {
                match code {
                    KeyCode::Char(c) => {
                        input.push(c);
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Enter => {
                        dispatch(
                            state,
                            cmd_tx,
                            WorkerCmd::Import {
                                name: input,
                                tar_path,
                                install_path: install_dir,
                            },
                        )
                        .await;
                        return ControlFlow::Continue(());
                    }
                    _ => {}
                }
                state.modal = Modal::ImportNameInput {
                    tar_path,
                    install_dir,
                    input,
                };
                ControlFlow::Continue(())
            }

            Modal::ImportInstallPicker {
                tar_path,
                mut explorer,
            } => match code {
                KeyCode::Esc => {
                    state.modal = Modal::None;
                    ControlFlow::Continue(())
                }
                KeyCode::Enter => {
                    let path = explorer.current().path.clone();
                    let install_dir = if path.is_file() {
                        path.parent().unwrap_or(&path).to_path_buf()
                    } else {
                        path
                    };

                    state.modal = Modal::ImportNameInput {
                        tar_path,
                        install_dir,
                        input: String::new(),
                    };
                    ControlFlow::Continue(())
                }
                _ => {
                    let key_event = KeyEvent::new(code, KeyModifiers::NONE);
                    let _ = explorer.handle(&Event::Key(key_event));
                    state.modal = Modal::ImportInstallPicker { tar_path, explorer };
                    ControlFlow::Continue(())
                }
            },

            Modal::ImportTarPicker { mut explorer } => match code {
                KeyCode::Esc => {
                    state.modal = Modal::None;
                    ControlFlow::Continue(())
                }
                KeyCode::Enter => {
                    let current = explorer.current();
                    if current.is_dir {
                        let key_event = KeyEvent::new(code, KeyModifiers::NONE);
                        let _ = explorer.handle(&Event::Key(key_event));
                        state.modal = Modal::ImportTarPicker { explorer };
                        ControlFlow::Continue(())
                    } else {
                        let tar_path = current.path.clone();
                        let mut next =
                            FileExplorerBuilder::build_with_theme(explorer_theme()).unwrap();
                        let _ = next.set_cwd(tar_path.parent().unwrap_or(&tar_path));
                        let _ = next.set_filter_map(|f| if f.is_dir { Some(f) } else { None });
                        state.modal = Modal::ImportInstallPicker {
                            tar_path,
                            explorer: next,
                        };
                        ControlFlow::Continue(())
                    }
                }
                _ => {
                    let key_event = KeyEvent::new(code, KeyModifiers::NONE);
                    let _ = explorer.handle(&Event::Key(key_event));
                    state.modal = Modal::ImportTarPicker { explorer };
                    ControlFlow::Continue(())
                }
            },

            Modal::ExportPicker {
                distros,
                mut explorer,
            } => match code {
                KeyCode::Esc => {
                    state.modal = Modal::None;
                    state.notify("Cancelled".to_string(), Level::Info, Anchor::TopRight, 2);
                    ControlFlow::Continue(())
                }
                KeyCode::Enter => {
                    let path = explorer.current().path.clone();
                    let export_dir = if path.is_file() {
                        path.parent().unwrap_or(&path).to_path_buf()
                    } else {
                        path
                    };

                    state.clear_multi_select();

                    let cmds = distros
                        .into_iter()
                        .map(|distro| {
                            let output = export_dir.join(format!("{distro}.tar"));
                            WorkerCmd::Export { distro, output }
                        })
                        .collect::<Vec<_>>();

                    dispatch(state, cmd_tx, WorkerCmd::Batch(cmds)).await;
                    ControlFlow::Continue(())
                }

                _ => {
                    let key_event = KeyEvent::new(code, KeyModifiers::NONE);
                    let _ = explorer.handle(&Event::Key(key_event));
                    state.modal = Modal::ExportPicker { distros, explorer };
                    ControlFlow::Continue(())
                }
            },

            Modal::CustomActionsMenu {
                distro,
                actions,
                mut selected,
            } => match code {
                KeyCode::Esc => {
                    state.modal = Modal::None;
                    ControlFlow::Continue(())
                }
                KeyCode::Up => {
                    selected = selected.saturating_sub(1);
                    state.modal = Modal::CustomActionsMenu {
                        distro,
                        actions,
                        selected,
                    };
                    ControlFlow::Continue(())
                }
                KeyCode::Down => {
                    if !actions.is_empty() {
                        selected = (selected + 1).min(actions.len() - 1);
                    }

                    state.modal = Modal::CustomActionsMenu {
                        distro,
                        actions,
                        selected,
                    };
                    ControlFlow::Continue(())
                }

                KeyCode::Enter => {
                    if let Some(action) = actions.get(selected) {
                        let action_name = action.name.clone();
                        let command = action.command.clone();

                        let (input_tx, input_rx) = mpsc::channel::<String>(32);

                        state.modal = Modal::ActionOutput {
                            distro: distro.clone(),
                            action_name: action_name.clone(),
                            output: format!("$ {command}\n"),
                            input: String::new(),
                            input_tx,
                            finished: false,
                        };

                        dispatch(
                            state,
                            cmd_tx,
                            WorkerCmd::RunCustomAction {
                                distro,
                                action_name,
                                command,
                                input_rx,
                            },
                        )
                        .await;
                    }
                    ControlFlow::Continue(())
                }

                _ => {
                    state.modal = Modal::CustomActionsMenu {
                        distro,
                        actions,
                        selected,
                    };
                    ControlFlow::Continue(())
                }
            },

            Modal::ActionOutput {
                distro,
                action_name,
                mut output,
                finished,
                mut input,
                input_tx,
            } => {
                let accepts_input = !finished;

                match code {
                    KeyCode::Esc | KeyCode::Char('q') if finished => {
                        state.modal = Modal::None;
                        ControlFlow::Continue(())
                    }

                    KeyCode::Char(c) if accepts_input => {
                        input.push(c);
                        state.modal = Modal::ActionOutput {
                            distro,
                            action_name,
                            output,
                            input_tx,
                            input,
                            finished,
                        };
                        ControlFlow::Continue(())
                    }

                    KeyCode::Backspace if accepts_input => {
                        input.pop();
                        state.modal = Modal::ActionOutput {
                            distro,
                            action_name,
                            output,
                            input_tx,
                            input,
                            finished,
                        };
                        ControlFlow::Continue(())
                    }

                    KeyCode::Enter if accepts_input => {
                        let submitted = format!("{input}\n");
                        let _ = input_tx.try_send(submitted);
                        output.push_str(&format!("\n> {}\n", "*".repeat(input.chars().count())));
                        state.modal = Modal::ActionOutput {
                            distro,
                            output,
                            input: String::new(),
                            input_tx,
                            finished,
                            action_name,
                        };
                        ControlFlow::Continue(())
                    }

                    KeyCode::Char('c') if finished => {
                        match arboard::Clipboard::new() {
                            Ok(mut clipboard) => {
                                let _ = clipboard.set_text(&output);
                                state.notify(
                                    "Output copied to clipboard".to_string(),
                                    Level::Info,
                                    Anchor::TopRight,
                                    2,
                                );
                            }
                            Err(e) => {
                                state.notify(
                                    format!("Clipboard unavailable: {e}"),
                                    Level::Error,
                                    Anchor::TopRight,
                                    2,
                                );
                            }
                        }
                        state.modal = Modal::ActionOutput {
                            distro,
                            action_name,
                            output,
                            finished,
                            input,
                            input_tx,
                        };
                        ControlFlow::Continue(())
                    }

                    _ => {
                        if !accepts_input {
                            input.clear();
                        }

                        state.modal = Modal::ActionOutput {
                            distro,
                            action_name,
                            output,
                            finished,
                            input,
                            input_tx,
                        };

                        ControlFlow::Continue(())
                    }
                }
            }
            Modal::CloneDistro {
                distro,
                mut new_name,
            } => match code {
                KeyCode::Esc => {
                    state.modal = Modal::None;
                    state.notify("Cancelled".to_string(), Level::Info, Anchor::TopRight, 2);
                    ControlFlow::Continue(())
                }

                KeyCode::Backspace => {
                    new_name.pop();
                    state.modal = Modal::CloneDistro { distro, new_name };
                    ControlFlow::Continue(())
                }

                KeyCode::Char(c) => {
                    new_name.push(c);
                    state.modal = Modal::CloneDistro { distro, new_name };
                    ControlFlow::Continue(())
                }

                KeyCode::Enter => {
                    let target = new_name.trim().to_string();

                    if target.is_empty() {
                        state.notify(
                            "Target name can't be empty".to_string(),
                            Level::Warn,
                            Anchor::TopRight,
                            4,
                        );
                        state.modal = Modal::CloneDistro { distro, new_name };
                        return ControlFlow::Continue(());
                    }

                    if target == distro {
                        state.notify(
                            "Clone name must be different".to_string(),
                            Level::Warn,
                            Anchor::TopRight,
                            4,
                        );
                        state.modal = Modal::CloneDistro { distro, new_name };
                        return ControlFlow::Continue(());
                    }

                    if state.distributions.iter().any(|d| d.name == target) {
                        state.notify(
                            format!("Distro `{target}` already exists"),
                            Level::Warn,
                            Anchor::TopRight,
                            3,
                        );
                        state.modal = Modal::CloneDistro { distro, new_name };
                        return ControlFlow::Continue(());
                    }

                    let base = config::config_dir().join("clones");
                    let export_dir = base.join("exports");
                    let distros_dir = base.join("distros");

                    if let Err(e) = fs::create_dir_all(&export_dir) {
                        state.notify(
                            "Failed to create export dir".to_string(),
                            Level::Error,
                            Anchor::TopRight,
                            4,
                        );
                        state.modal = Modal::CloneDistro { distro, new_name };
                        return ControlFlow::Continue(());
                    };

                    if let Err(e) = fs::create_dir_all(&distros_dir) {
                        state.notify(
                            "Failed to create distros dir".to_string(),
                            Level::Error,
                            Anchor::TopRight,
                            4,
                        );
                        state.modal = Modal::CloneDistro { distro, new_name };
                        return ControlFlow::Continue(());
                    };

                    let ts = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    let tar_path = export_dir.join(format!("{distro}-clone-{target}-{ts}.tar"));

                    let install_path = distros_dir.join(&target);

                    if let Err(e) = fs::create_dir_all(&install_path) {
                        state.notify(
                            format!("Failed to create install dir: {e}"),
                            Level::Error,
                            Anchor::TopRight,
                            4,
                        );
                        state.modal = Modal::CloneDistro { distro, new_name };
                        return ControlFlow::Continue(());
                    }

                    state.modal = Modal::None;

                    dispatch(
                        state,
                        cmd_tx,
                        WorkerCmd::Batch(vec![
                            WorkerCmd::Export {
                                distro: distro.clone(),
                                output: tar_path.clone(),
                            },
                            WorkerCmd::Import {
                                name: target,
                                tar_path,
                                install_path,
                            },
                            WorkerCmd::Refresh,
                        ]),
                    )
                    .await;
                    ControlFlow::Continue(())
                }
                _ => {
                    state.modal = Modal::CloneDistro { distro, new_name };
                    ControlFlow::Continue(())
                }
            },
            Modal::RollBackDistroPicker {
                distros,
                mut selected,
            } => match code {
                KeyCode::Esc => {
                    state.modal = Modal::None;
                    ControlFlow::Continue(())
                }

                KeyCode::Up => {
                    selected = selected.saturating_sub(1);
                    state.modal = Modal::RollBackDistroPicker { distros, selected };
                    ControlFlow::Continue(())
                }

                KeyCode::Down => {
                    if !distros.is_empty() {
                        selected = (selected + 1).min(distros.len() - 1);
                    }
                    state.modal = Modal::RollBackDistroPicker { distros, selected };
                    ControlFlow::Continue(())
                }

                KeyCode::Enter => {
                    if let Some(distro) = distros.get(selected).clone() {
                        let snaps = snapshots::list_snapshots_for_distro(&distro);
                        if snaps.is_empty() {
                            state.notify(
                                format!("No snapshots found for {distro}"),
                                Level::Info,
                                Anchor::TopRight,
                                2,
                            );
                            state.modal = Modal::RollBackDistroPicker { distros, selected };
                            return ControlFlow::Continue(());
                        }
                        state.modal = Modal::RollBackSnapShotPicker {
                            distro: distro.clone(),
                            snapshots: snaps,
                            selected: 0,
                        };
                    }
                    ControlFlow::Continue(())
                }

                _ => {
                    state.modal = Modal::RollBackDistroPicker { distros, selected };
                    ControlFlow::Continue(())
                }
            },
            Modal::RollBackSnapShotPicker {
                distro,
                snapshots,
                mut selected,
            } => match code {
                KeyCode::Esc => {
                    state.modal = Modal::None;
                    ControlFlow::Continue(())
                }

                KeyCode::Up => {
                    selected = selected.saturating_sub(1);
                    state.modal = Modal::RollBackSnapShotPicker {
                        distro,
                        snapshots,
                        selected,
                    };
                    ControlFlow::Continue(())
                }

                KeyCode::Down => {
                    if !snapshots.is_empty() {
                        selected = (selected + 1).min(snapshots.len() - 1);
                    }
                    state.modal = Modal::RollBackSnapShotPicker {
                        distro,
                        snapshots,
                        selected,
                    };
                    ControlFlow::Continue(())
                }

                KeyCode::Enter => {
                    if let Some(snapshot) = snapshots.get(selected).cloned() {
                        let exists = state.distributions.iter().any(|d| d.name == distro);
                        state.modal = Modal::ConfirmRollBack {
                            distro,
                            snapshot,
                            exists,
                        };
                    } else {
                        state.modal = Modal::RollBackSnapShotPicker {
                            distro,
                            snapshots,
                            selected,
                        };
                    }
                    ControlFlow::Continue(())
                }
                _ => {
                    state.modal = Modal::RollBackSnapShotPicker {
                        distro,
                        snapshots,
                        selected,
                    };
                    ControlFlow::Continue(())
                }
            },
            Modal::ConfirmRollBack {
                distro,
                snapshot,
                exists,
            } => match code {
                KeyCode::Char('y') => {
                    let install_path = state
                        .distributions
                        .iter()
                        .find(|d| d.name == distro)
                        .and_then(|d| d.install_path.as_deref().map(PathBuf::from))
                        .unwrap_or_else(|| config::config_dir().join("restores").join(&distro));

                    if let Err(e) = std::fs::create_dir_all(&install_path) {
                        state.notify(
                            format!("Failed to create install dir: {e}"),
                            Level::Error,
                            Anchor::TopRight,
                            3,
                        );
                        return ControlFlow::Continue(());
                    }

                    let mut cmds = vec![WorkerCmd::Shutdown];
                    if exists {
                        cmds.push(WorkerCmd::Unregister(distro.clone()));
                    }
                    cmds.push(WorkerCmd::Import {
                        name: distro.clone(),
                        tar_path: snapshot,
                        install_path,
                    });
                    cmds.push(WorkerCmd::Refresh);

                    dispatch(state, cmd_tx, WorkerCmd::Batch(cmds)).await;
                    ControlFlow::Continue(())
                }

                KeyCode::Char('n') => {
                    state.notify("Cancelled".to_string(), Level::Info, Anchor::TopRight, 2);
                    ControlFlow::Continue(())
                }

                KeyCode::Char('q') => ControlFlow::Break(()),

                _ => {
                    state.modal = Modal::ConfirmRollBack {
                        distro,
                        snapshot,
                        exists,
                    };
                    ControlFlow::Continue(())
                }
            },

            Modal::None => ControlFlow::Continue(()),
        }
    }
}

impl Component for ModalComponent {
    fn handle_event<'a>(
        &'a mut self,
        state: &'a mut AppState,
        cmd_tx: &'a mpsc::Sender<WorkerCmd>,
        ev: Event,
    ) -> Pin<Box<dyn Future<Output = ControlFlow<()>> + '_>> {
        Box::pin(self.handle_event_inner(state, cmd_tx, ev))
    }
}
