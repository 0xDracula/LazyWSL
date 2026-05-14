use crate::app::worker::{spawn_wsl_worker, WorkerCmd, WorkerEvent};
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::ops::ControlFlow;
use crossterm::event::{KeyEvent, KeyModifiers};
use ratatree::{FilePickerState, PickerResult};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc::{self, Sender};
pub use state::{AppState, Pending};
use crate::ui;
use tokio_stream::StreamExt;
use crate::app::Pending::ImportNameInput;

mod state;
mod worker;

pub struct App {
    cmd_tx: Sender<WorkerCmd>,
    pub state: AppState,
}

impl App {
    pub fn new(cmd_tx: Sender<WorkerCmd>) -> Self {
        Self {
            cmd_tx,
            state: AppState::default(),
        }
    }

    fn apply_worker_event(&mut self, ev: WorkerEvent) {
        match ev {
            WorkerEvent::DistroUpdated {
                distributions,
                status_line,
            } => {
                self.state.busy = false;
                self.state.status_line = status_line;
                if let Ok(v) = distributions {
                    self.state.distributions = v;
                }
                self.state.clamp_selection();
            }
            WorkerEvent::ListOnly { distributions } => {
                if let Ok(v) = distributions {
                    self.state.distributions = v;
                }
                self.state.clamp_selection();
            }
        }
    }

    async fn dispatch(&mut self, cmd: WorkerCmd) {
        self.state.busy = true;
        self.state.status_line = "Working..".to_string();
        if self.cmd_tx.send(cmd).await.is_err() {
            self.state.busy = false;
            self.state.status_line = "WSL worker stopped.".to_string();
        }
    }

    pub async fn handle_event(&mut self, ev: Event) -> ControlFlow<()> {
        match ev {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Release {
                    ControlFlow::Continue(())
                } else {
                    self.handle_key(key.code).await
                }
            }
            Event::Resize(_, _)
            | Event::FocusLost
            | Event::FocusGained
            | Event::Paste(_)
            | Event::Mouse(_) => ControlFlow::Continue(()),
        }
    }

    pub async fn handle_key(&mut self, code: KeyCode) -> ControlFlow<()> {
        if self.state.busy {
            return ControlFlow::Continue(());
        }

        if !matches!(self.state.pending, Pending::None) {
            return self.handle_pending(code).await
        }

        match code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return ControlFlow::Break(()),
            KeyCode::Esc => return ControlFlow::Break(()),
            KeyCode::Char('h') => {
                self.state.pending = Pending::Help;
            },
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if let Some(d) = self.state.selected_distro().map(|d| d.name.clone()) {
                    self.dispatch(WorkerCmd::RunDistro(d)).await;
                }
            }
            KeyCode::Down => self.move_selection(1),
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Enter => {
                if let Some(d) = self.state.selected_distro().map(|d| d.name.clone()) {
                    self.dispatch(WorkerCmd::OpenShell(d)).await;
                }
            }
            KeyCode::Char('t') => {
                if let Some(d) = self.state.selected_distro().map(|d| d.name.clone()) {
                    self.dispatch(WorkerCmd::Terminate(d)).await;
                }
            }
            KeyCode::Char('d') => {
                if let Some(d) = self.state.selected_distro().map(|d| d.name.clone()) {
                    self.dispatch(WorkerCmd::SetDefault(d)).await;
                }
            }
            KeyCode::Char('u') => {
                if let Some(d) = self.state.selected_distro().map(|d| d.name.clone()) {
                    self.state.pending = Pending::ConfirmUnregister { name: d };
                    self.state.status_line = "Unregister is destructive, press y to confirm!".to_string();
                }
            }
            KeyCode::Char('s') => {
                    self.state.pending = Pending::ConfirmShutdown;
                    self.state.status_line = "Shutdown stops all WSL2 VMs, press y to confirm!".to_string();
            }
            KeyCode::Char('e') => {
                if let Some(distro) = self.state.selected_distro() {
                    self.state.pending = Pending::ExportPicker { distro: distro.name.clone(), picker: FilePickerState::builder()
                        .start_dir(std::env::current_dir().unwrap())
                        .mode(ratatree::PickerMode::DirsOnly)
                        .build() }
                }
            }
            KeyCode::Char('i') => {
                self.state.pending = Pending::ImportTarPicker {
                    picker: FilePickerState::builder().start_dir(std::env::current_dir().unwrap()).build(),
                };
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }

    fn move_selection(&mut self, delta: isize) {
        if self.state.distributions.is_empty() {
            return;
        }

        let len = self.state.distributions.len();
        let i = self.state.selected as isize + delta;
        let i = i.clamp(0, (len - 1) as isize) as usize;
        self.state.selected = i;
    }

    async fn handle_pending(&mut self, code: KeyCode) -> ControlFlow<()> {
        let pending = std::mem::replace(&mut self.state.pending, Pending::None);
        match pending {
            Pending::None => ControlFlow::Continue(()),
            Pending::ConfirmUnregister { name } => match code {
                KeyCode::Char('y') => {
                    self.state.pending = Pending::None;
                    self.dispatch(WorkerCmd::Unregister(name)).await;
                    ControlFlow::Continue(())
                }
                KeyCode::Char('n') => {
                    self.state.pending = Pending::None;
                    self.state.status_line = "Cancelled".to_string();
                    ControlFlow::Continue(())
                }
                KeyCode::Char('q') => ControlFlow::Break(()),
                _ => ControlFlow::Continue(()),
            },
            Pending::ConfirmShutdown => match code {
                KeyCode::Char('y') => {
                    self.state.pending = Pending::None;
                    self.dispatch(WorkerCmd::Shutdown).await;
                    ControlFlow::Continue(())
                }
                KeyCode::Char('n') => {
                    self.state.pending = Pending::None;
                    self.state.status_line = "Cancelled".to_string();
                    ControlFlow::Continue(())
                }
                KeyCode::Char('q') => ControlFlow::Break(()),
                _ => ControlFlow::Continue(()),
            },
            Pending::Help => match code {
                KeyCode::Esc => {
                    self.state.pending = Pending::None;
                    ControlFlow::Continue(())
                }
                _ => ControlFlow::Continue(())
            },
            ImportNameInput { tar_path, install_dir, mut input } => {
                match code {
                    KeyCode::Char(c) => {
                        input.push(c);
                        self.state.pending = ImportNameInput {
                            tar_path,
                            install_dir,
                            input,
                        };
                        ControlFlow::Continue(())
                    }
                    KeyCode::Backspace => {
                        input.pop();

                        self.state.pending = ImportNameInput {
                            tar_path,
                            install_dir,
                            input,
                        };
                        ControlFlow::Continue(())
                    }
                    KeyCode::Enter => {
                        self.dispatch(WorkerCmd::Import { name: input, tar_path, install_path: install_dir }).await;
                        self.state.pending = Pending::None;
                        ControlFlow::Continue(())
                    }
                    _ => {
                        self.state.pending = ImportNameInput {
                            tar_path,
                            install_dir,
                            input,
                        };
                        ControlFlow::Continue(())
                    }
                }
            }
            Pending::ImportInstallPicker {
                tar_path,
                mut picker,
            } => {
                let key_event = KeyEvent::new(code, KeyModifiers::NONE);
                picker.handle_event(Event::Key(key_event));
                match picker.result() {
                    PickerResult::Pending => {
                        self.state.pending = Pending::ImportInstallPicker {
                            tar_path,
                            picker,
                        };
                        ControlFlow::Continue(())
                    }
                    PickerResult::Cancelled => {
                        self.state.pending = Pending::None;
                        ControlFlow::Continue(())
                    }
                    PickerResult::Selected(paths) => {
                        if let Some(path) = paths.first() {
                            let install_dir = if path.is_file() {
                                path.parent().unwrap_or(path).to_path_buf()
                            } else {
                                path.clone()
                            };

                            self.state.pending = ImportNameInput { tar_path, install_dir, input: String::new() };
                        }
                        ControlFlow::Continue(())
                    }
                }

            },
            Pending::ImportTarPicker {
              mut picker,
            } => {
                let key_event = KeyEvent::new(code, KeyModifiers::NONE);
                picker.handle_event(Event::Key(key_event));
                match picker.result() {
                    PickerResult::Pending => {
                        self.state.pending = Pending::ImportTarPicker {picker};
                        ControlFlow::Continue(())
                    }
                    PickerResult::Cancelled => {
                        self.state.pending = Pending::None;
                        ControlFlow::Continue(())
                    }
                    PickerResult::Selected(paths) => {
                        if let Some(path) = paths.first() {
                            self.state.pending = Pending::ImportInstallPicker {
                                tar_path: path.clone(),
                                picker: FilePickerState::builder().start_dir(std::env::current_dir().unwrap()).build()
                            };
                        }
                        ControlFlow::Continue(())
                    }
                }
            }
            Pending::ExportPicker {
                distro,
                mut picker,
            } => {
                let key_event = KeyEvent::new(
                    code,
                    KeyModifiers::NONE,
                );

                picker.handle_event(Event::Key(key_event));

                match picker.result() {
                    PickerResult::Pending => {
                        self.state.pending = Pending::ExportPicker {
                            distro,
                            picker,
                        };

                        ControlFlow::Continue(())
                    },
                    PickerResult::Cancelled => {
                        self.state.pending = Pending::None;
                        self.state.status_line = "Export Cancelled".to_string();
                        ControlFlow::Continue(())
                    },
                    PickerResult::Selected(paths) => {
                        if let Some(path) = paths.first() {
                            let export_dir = if path.is_file() {
                                path.parent().unwrap_or(path)
                            } else {
                                path
                            };
                            let output = export_dir.join(format!("{distro}.tar"));
                            self.dispatch(WorkerCmd::Export { distro, output, }).await;
                        }
                        self.state.pending = Pending::None;
                        ControlFlow::Continue(())
                    }
                }
            }
        }
    }
}

pub async fn run_tui() -> io::Result<()> {
    let (cmd_tx, cmd_rx) = mpsc::channel::<WorkerCmd>(32);
    let (evt_tx, mut evt_rx) = mpsc::channel::<WorkerEvent>(128);

    let _worker = spawn_wsl_worker(cmd_rx, evt_tx);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new(cmd_tx);
    app.dispatch(WorkerCmd::Refresh).await;

    terminal.draw(|f| ui::render(f, &mut app.state))?;

    let mut events = EventStream::new();

    let run_inner = async {
        loop {
            tokio::select! {
                biased;

                Some(ev) = evt_rx.recv() => {
                    app.apply_worker_event(ev);
                    terminal.draw(|f| ui::render(f, &mut app.state))?;
                }

                reader = events.next() => {
                    match reader {
                        Some(Ok(ev)) => {
                            if matches!(&ev, Event::Resize(_, _)) {
                                terminal.draw(|f| ui::render(f, &mut app.state))?;
                                continue;
                            }
                            match app.handle_event(ev).await {
                                ControlFlow::Break(()) => return Ok::<(), io::Error>(()),
                                ControlFlow::Continue(()) => {
                                    terminal.draw(|f| ui::render(f, &mut app.state))?;
                                }
                            }
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

    drop(app);

    result
}