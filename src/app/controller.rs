use crate::app::components::modal::ModalComponent;
use crate::app::keymaps::map_key;
use crate::app::reducers::reduce;
use crate::app::worker::commands::{WorkerCmd, WorkerEvent};
use crate::app::worker::runner::spawn_wsl_worker;
use crate::app::{AppState, Modal};
use crate::ui;
use crate::ui::Component;
use crate::ui::{Anchor, Level};
use crate::wsl::{MockWSLService, WSLProcessService, WSLService};
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::ops::ControlFlow;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use tokio_stream::StreamExt;

fn make_backend() -> std::sync::Arc<dyn WSLService> {
    let force_mock = std::env::var("LAZYWSL_MOCK")
        .map(|v| v != "0" && !v.is_empty())
        .unwrap_or(false);

    if cfg!(windows) && !force_mock {
        std::sync::Arc::new(WSLProcessService::new())
    } else {
        std::sync::Arc::new(MockWSLService::new())
    }
}

pub async fn run_tui() -> io::Result<()> {
    let (cmd_tx, cmd_rx) = mpsc::channel::<WorkerCmd>(32);
    let (evt_tx, mut evt_rx) = mpsc::channel::<WorkerEvent>(128);

    let wsl = make_backend();
    let _worker = spawn_wsl_worker(cmd_rx, evt_tx, wsl);

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut state = AppState::default();

    terminal.draw(|f| ui::render(f, &mut state))?;

    dispatch(&mut state, &cmd_tx, WorkerCmd::Refresh).await;

    let mut events = EventStream::new();

    let mut tick = time::interval(Duration::from_millis(16));

    let run_inner = async {
        loop {
            tokio::select! {
                biased;

                Some(ev) = evt_rx.recv() => {
                    apply_worker_event(&mut state, ev);
                }

                reader = events.next() => {
                    match reader {
                        Some(Ok(ev)) => {
                            if handle_event(&mut state, &cmd_tx, ev).await == ControlFlow::Break(()) {
                                return Ok::<(), io::Error>(());
                            }
                        }
                        Some(Err(e)) => return Err(e),
                        None => return Ok::<(), io::Error>(()),
                    }
                }
                _ = tick.tick() => {}
            }

            state.notifications.tick();
            terminal.draw(|f| ui::render(f, &mut state))?;
        }
    };
    let result = run_inner.await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    result
}

fn apply_worker_event(state: &mut AppState, ev: WorkerEvent) {
    match ev {
        WorkerEvent::StateRefresh {
            distributions,
            status_line,
        } => {
            state.busy = false;
            if let Some(msg) = status_line {
                state.notify(msg, Level::Info, Anchor::TopRight, 2);
            }
            if let Ok(v) = distributions {
                state.distributions = v;
            }
            state.clamp_selection();
        }
        WorkerEvent::CustomActionOutput { chunk } => {
            if let Modal::ActionOutput { output, .. } = &mut state.modal {
                output.push_str(&chunk);
            }
        }
        WorkerEvent::CustomActionFinished { status_line } => {
            state.busy = false;
            state.notify(status_line.clone(), Level::Info, Anchor::TopRight, 2);

            if let Modal::ActionOutput {
                output, finished, ..
            } = &mut state.modal
            {
                output.push('\n');
                output.push_str(&status_line);
                output.push('\n');
                *finished = true;
            }
        }
        WorkerEvent::CatalogFetched { entries } => match entries {
            Ok(list) => {
                let filtered = (0..list.len()).collect();
                state.modal = Modal::CatalogPicker {
                    entries: list,
                    filtered,
                    selected: 0,
                    query: String::new(),
                };
            }
            Err(e) => {
                state.modal = Modal::None;
                state.notify(
                    format!("Catalog failed: {e}"),
                    Level::Error,
                    Anchor::TopRight,
                    3,
                )
            }
        },
    }
}

pub async fn dispatch(state: &mut AppState, tx: &mpsc::Sender<WorkerCmd>, cmd: WorkerCmd) {
    state.busy = true;
    if tx.send(cmd).await.is_err() {
        state.busy = false;
        state.notify(
            "WSL worker has stopped".to_string(),
            Level::Error,
            Anchor::TopRight,
            2,
        );
    }
}

async fn handle_event(
    state: &mut AppState,
    cmd_tx: &mpsc::Sender<WorkerCmd>,
    ev: Event,
) -> ControlFlow<()> {
    let mut model_component = ModalComponent::new();

    match ev {
        Event::Key(key) => {
            if key.kind == KeyEventKind::Release {
                return ControlFlow::Continue(());
            }
            if !matches!(state.modal, Modal::None) {
                return model_component
                    .handle_event(state, cmd_tx, Event::Key(key))
                    .await;
            }
            if state.search_active {
                return handle_search_key(state, key.code);
            }
            if state.busy {
                return ControlFlow::Continue(());
            }
            let action = map_key(key, &state.keymaps);
            let cmds = reduce(state, action);
            for cmd in cmds {
                dispatch(state, cmd_tx, cmd).await;
            }

            if state.should_quit {
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        }
        _ => ControlFlow::Continue(()),
    }
}

fn handle_search_key(state: &mut AppState, code: KeyCode) -> ControlFlow<()> {
    match code {
        KeyCode::Esc | KeyCode::Enter => {
            state.search_active = false;
            let msg = if state.search_query.is_empty() {
                "Search Cleared".to_string()
            } else {
                format!("Search: {}", state.search_query)
            };

            state.notify(msg, Level::Info, Anchor::TopRight, 2);
        }
        KeyCode::Backspace => {
            state.search_query.pop();
        }
        KeyCode::Char(c) => {
            state.search_query.push(c);
        }
        KeyCode::Up => state.move_selection(-1),
        KeyCode::Down => state.move_selection(1),
        _ => {}
    }
    state.selected = 0;
    state.clamp_selection();
    ControlFlow::Continue(())
}
