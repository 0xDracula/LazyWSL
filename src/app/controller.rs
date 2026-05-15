use std::io;
use std::ops::ControlFlow;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatree::FilePickerState;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use ratatui_explorer::FileExplorer;
use tokio_stream::StreamExt;
use tokio::sync::mpsc::{self, Sender};
use crate::app::{AppState, Modal};
use crate::app::actions::map_key;
use crate::app::reducers::reduce;
use crate::app::worker::commands::{WorkerCmd, WorkerEvent};
use crate::app::worker::runner::spawn_wsl_worker;
use crate::ui;
use crate::wsl::WSLProcessService;

pub async fn run_tui() -> io::Result<()> {
    let (cmd_tx, cmd_rx) = mpsc::channel::<WorkerCmd>(32);
    let (evt_tx, mut evt_rx) = mpsc::channel::<WorkerEvent>(128);

    let wsl = std::sync::Arc::new(WSLProcessService::new());
    let _worker = spawn_wsl_worker(cmd_rx, evt_tx, wsl);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut state = AppState::default();
    // TODO! Dispatch

    terminal.draw(|f| ui::render(f, &mut state))?;

    let mut events = EventStream::new();

    let run_inner = async {
        loop {
            tokio::select! {
                biased;

                Some(ev) = evt_rx.recv() => {
                    apply_worker_event(&mut state, ev);
                    terminal.draw(|f| ui::render(f, &mut state))?;
                }

                reader = events.next() => {
                    match reader {
                        Some(Ok(ev)) => {
                            if matches!(&ev, Event::Resize(_,_)) {
                                terminal.draw(|f| ui::render(f, &mut state))?;
                                continue;
                            }
                            if handle_event(&mut state, &cmd_tx, ev).await == ControlFlow::Break(()) {
                                return Ok::<(), io::Error>(());
                            }
                            terminal.draw(|f| ui::render(f, &mut state))?;
                        }
                        Some(Err(e)) => return Err(e),
                        None => return Ok::<(), io::Error>(()),
                    }
                }
            }
        }
    };

    let result = run_inner.await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    result
}

fn apply_worker_event(state: &mut AppState, ev: WorkerEvent) {
    match ev {
        WorkerEvent::DistroUpdated {
            distributions,
            status_line,
        } => {
            state.busy = false;
            state.status_line = status_line;
            if let Ok(v) = distributions { state.distributions = v; }
            state.clamp_selection();
        }
        WorkerEvent::ListOnly { distributions } => {
            if let Ok(v) = distributions { state.distributions = v; }
            state.clamp_selection()
        }
    }
}

async fn dispatch(state: &mut AppState, tx: &mpsc::Sender<WorkerCmd>, cmd: WorkerCmd) {
    state.busy = true;
    state.status_line = "Working...".to_string();
    if tx.send(cmd).await.is_err() {
        state.busy = false;
        state.status_line = "WSL worker has stopped".to_string();
    }
}

async fn handle_event(state: &mut AppState, cmd_tx: &mpsc::Sender<WorkerCmd>, ev: Event) -> ControlFlow<()> {
    match ev {
        Event::Key(key) => {
            if key.kind == KeyEventKind::Release { return ControlFlow::Continue(()); }
            if state.busy { return ControlFlow::Continue(()); }
            if !matches!(state.modal, Modal::None) {
                return handle_modal_key(state, cmd_tx, key.code).await;
            }

            let action = map_key(key.code);
            if let Some(cmd) = reduce(state, action) {
                dispatch(state, cmd_tx, cmd).await;
            }

            if state.should_quit { return ControlFlow::Break(()); }
            ControlFlow::Continue(())
        }
        _ => ControlFlow::Continue(()),
    }
}

async fn handle_modal_key(state: &mut AppState, cmd_tx: &mpsc::Sender<WorkerCmd>, code: KeyCode) -> ControlFlow<()> {
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

        Modal::ConfirmUnregister { name } => match code {
            KeyCode::Char('y') => {
                dispatch(state, cmd_tx, WorkerCmd::Unregister(name)).await;
                ControlFlow::Continue(())
            }
            KeyCode::Char('n') => {
                state.status_line = "Cancelled.".to_string();
                ControlFlow::Continue(())
            }
            KeyCode::Char('q') => ControlFlow::Break(()),
            _ => {
                state.modal = Modal::ConfirmUnregister { name };
                ControlFlow::Continue(())
            }
        }

        Modal::ConfirmShutdown => match code {
            KeyCode::Char('y') => {
                dispatch(state, cmd_tx, WorkerCmd::Shutdown).await;
                ControlFlow::Continue(())
            }
            KeyCode::Char('n') => {
                state.status_line = "Cancelled".to_string();
                ControlFlow::Continue(())
            }
            KeyCode::Char('q') => {
                ControlFlow::Break(())
            }
            _ => {
                state.modal = Modal::ConfirmShutdown;
                ControlFlow::Continue(())
            }
        }

        Modal::ImportNameInput { tar_path, install_dir, mut input } => {
            match code {
                KeyCode::Char(c) => { input.push(c); }
                KeyCode::Backspace => { input.pop(); }
                KeyCode::Enter => {
                    dispatch(state, cmd_tx, WorkerCmd::Import { name: input, tar_path, install_path: install_dir }).await;
                    return ControlFlow::Continue(());
                }
                _ => {}
            }
            state.modal = Modal::ImportNameInput { tar_path, install_dir, input };
            ControlFlow::Continue(())
        }

        Modal::ImportInstallPicker { tar_path, mut explorer } => {
            match code {
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
                    state.modal = Modal::ImportNameInput { tar_path, install_dir, input: String::new() };
                    ControlFlow::Continue(())
                }
                _ => {
                    let key_event = KeyEvent::new(code, KeyModifiers::NONE);
                    let _ = explorer.handle(&Event::Key(key_event));
                    state.modal = Modal::ImportInstallPicker { tar_path, explorer };
                    ControlFlow::Continue(())
                }
            }
        }

        Modal::ImportTarPicker { mut explorer } => {
            match code {
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
                        let mut next = FileExplorer::new().expect("Failed to create file explorer!");
                        let _ = next.set_cwd(tar_path.parent().unwrap_or(&tar_path));
                        let _ = next.set_filter_map(|f| if f.is_dir { Some(f) } else { None });
                        state.modal = Modal::ImportInstallPicker { tar_path, explorer: next };
                        ControlFlow::Continue(())
                    }
                }
                _ => {
                    let key_event = KeyEvent::new(code, KeyModifiers::NONE);
                    let _ = explorer.handle(&Event::Key(key_event));
                    state.modal = Modal::ImportTarPicker { explorer };
                    ControlFlow::Continue(())
                }
            }
        }

        Modal::ExportPicker { distro, mut explorer } => {
            match code {
                KeyCode::Esc => {
                    state.modal = Modal::None;
                    state.status_line = "Cancelled..".to_string();
                    ControlFlow::Continue(())
                }
                KeyCode::Enter => {
                    let path = explorer.current().path.clone();
                    let export_dir = if path.is_file() {
                        path.parent().unwrap_or(&path).to_path_buf()
                    } else {
                        path
                    };

                    let output = export_dir.join(format!("{distro}.tar"));
                    dispatch(state, cmd_tx, WorkerCmd::Export { distro, output }).await;
                    ControlFlow::Continue(())
                }
                _ => {
                    let key_event = KeyEvent::new(code, KeyModifiers::NONE);
                    let _ = explorer.handle(&Event::Key(key_event));
                    state.modal = Modal::ExportPicker { distro, explorer };
                    ControlFlow::Continue(())
                }
            }
        }

        Modal::None => ControlFlow::Continue(())
    }
}