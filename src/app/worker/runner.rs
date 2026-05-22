use crate::app::worker::commands::{WorkerCmd, WorkerEvent};
use crate::config;
use crate::core::WSLError;
use crate::wsl::WSLService;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{self, MissedTickBehavior};

pub fn spawn_wsl_worker(
    cmd_rx: Receiver<WorkerCmd>,
    evt_tx: Sender<WorkerEvent>,
    wsl: Arc<dyn WSLService>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        run_wsl_worker(cmd_rx, evt_tx, wsl).await;
    })
}

fn friendly_status(context: &str, err: &WSLError) -> String {
    match err {
        WSLError::NotInstalled => {
            "WSL isn't installed, run `wsl --install`, reboot, then install a distro from Microsoft store".to_string()
        }
        WSLError::NoDistros => {
            "No distros found, install a distro from microsoft store ".to_string()
        }

        WSLError::ProcessFailed { code, stderr } => {
            if stderr.is_empty() {
                format!{"{context}: Exit code {code}"}
            } else {
                format!("{context}: {stderr} (Code {code})")
            }
        }
        _ => {
            format!("{context}: {err}")
        }
    }
}

async fn run_wsl_worker(
    mut cmd_rx: tokio::sync::mpsc::Receiver<WorkerCmd>,
    evt_tx: Sender<WorkerEvent>,
    wsl: Arc<dyn WSLService>,
) {
    let mut tick = time::interval_at(
        time::Instant::now() + Duration::from_secs(config::load_or_create().refresh_secs),
        Duration::from_secs(config::load_or_create().refresh_secs),
    );
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            biased;

            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(cmd) => { process_cmd(&wsl, &evt_tx, cmd).await },
                    None => break,
                }
            }
            _ = tick.tick() => {
                let list = wsl.list().await;
                if evt_tx.send(WorkerEvent::StateRefresh { distributions: list, status_line: None }).await.is_err() {
                    break;
                }
            }
        }
    }
}

fn process_cmd<'a>(
    wsl: &'a Arc<dyn WSLService>,
    evt_tx: &'a Sender<WorkerEvent>,
    cmd: WorkerCmd,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
    Box::pin(async move {
        if let WorkerCmd::Batch(cmds) = cmd {
            for c in cmds {
                process_cmd(wsl, evt_tx, c).await;
            }
            return;
        }

        if let WorkerCmd::RunCustomAction {
            distro,
            action_name,
            command,
            input_rx,
        } = cmd
        {
            process_custom_action(wsl, evt_tx, distro, action_name, command, input_rx).await;
            return;
        }
        let evt = match cmd {
            WorkerCmd::Refresh => {
                let distributions = wsl.list().await;
                let status_line = match &distributions {
                    Ok(v) => format!("Loaded {} distro(s).", v.len()),
                    Err(e) => friendly_status("Refresh failed", e),
                };
                WorkerEvent::StateRefresh {
                    distributions,
                    status_line: Some(status_line),
                }
            }
            WorkerCmd::RunDistro(name) => {
                let op = wsl.run(&name).await;
                let distributions = wsl.list().await;
                let status_line = match op {
                    Ok(()) => format!("Ran distro `{name}`."),
                    Err(e) => friendly_status("Run distro failed.", &e),
                };
                WorkerEvent::StateRefresh {
                    distributions,
                    status_line: Some(status_line),
                }
            }
            WorkerCmd::Terminate(name) => {
                let op = wsl.terminate(&name).await;
                let distributions = wsl.list().await;
                let status_line = match op {
                    Ok(()) => format!("Terminated `{name}`."),
                    Err(e) => friendly_status("Terminate Failed", &e),
                };
                WorkerEvent::StateRefresh {
                    distributions,
                    status_line: Some(status_line),
                }
            }
            WorkerCmd::Unregister(name) => {
                let op = wsl.unregister(&name).await;
                let distributions = wsl.list().await;
                let status_line = match op {
                    Ok(()) => format!("Unregistered `{name}`."),
                    Err(e) => friendly_status("Failed to Unregister", &e),
                };
                WorkerEvent::StateRefresh {
                    distributions,
                    status_line: Some(status_line),
                }
            }
            WorkerCmd::SetDefault(name) => {
                let op = wsl.set_default(&name).await;
                let distributions = wsl.list().await;
                let status_line = match op {
                    Ok(()) => format!("`{name}` Set Default Successfully."),
                    Err(e) => friendly_status("Failed to Set Default", &e),
                };
                WorkerEvent::StateRefresh {
                    distributions,
                    status_line: Some(status_line),
                }
            }
            WorkerCmd::Shutdown => {
                let op = wsl.shutdown().await;
                let distributions = wsl.list().await;
                let status_line = match op {
                    Ok(()) => "Shutdown Succesful".to_string(),
                    Err(e) => friendly_status("Shutdown failed", &e),
                };
                WorkerEvent::StateRefresh {
                    distributions,
                    status_line: Some(status_line),
                }
            }
            WorkerCmd::OpenShell(name) => {
                let op = wsl.open_shell(&name).await;
                let distributions = wsl.list().await;
                let status_line = match op {
                    Ok(()) => format!("Opened shell for `{name}`"),
                    Err(e) => friendly_status("Open Shell Failed", &e),
                };
                WorkerEvent::StateRefresh {
                    distributions,
                    status_line: Some(status_line),
                }
            }
            WorkerCmd::Import {
                name,
                tar_path,
                install_path,
            } => {
                let op = wsl.import(&name, &tar_path, &install_path).await;
                let distributions = wsl.list().await;
                let status_line = match op {
                    Ok(()) => format!("Imported `{name}`"),
                    Err(e) => friendly_status("Import failed", &e),
                };
                WorkerEvent::StateRefresh {
                    distributions,
                    status_line: Some(status_line),
                }
            }
            WorkerCmd::Export { distro, output } => {
                let op = wsl.export(&distro, &output).await;
                let distributions = wsl.list().await;
                let status_line = match op {
                    Ok(()) => format!("Exported {distro}"),
                    Err(e) => friendly_status("Export Failed", &e),
                };
                WorkerEvent::StateRefresh {
                    distributions,
                    status_line: Some(status_line),
                }
            }
            WorkerCmd::RunCustomAction { .. } | WorkerCmd::Batch(_) => unreachable!(),
        };

        let _ = evt_tx.send(evt).await;
    })
}

async fn process_custom_action(
    wsl: &Arc<dyn WSLService>,
    evt_tx: &Sender<WorkerEvent>,
    distro: String,
    action_name: String,
    command: String,
    input_rx: Receiver<String>,
) {
    let (line_tx, mut line_rx) = tokio::sync::mpsc::channel::<String>(128);
    let op = wsl.run_custom_action(&distro, &command, line_tx, input_rx);
    tokio::pin!(op);

    let mut output_open = true;
    let result = loop {
        tokio::select! {
            chunk = line_rx.recv(), if output_open => {
                match chunk {
                    Some(chunk) => {
                        if evt_tx.send(WorkerEvent::CustomActionOutput { chunk }).await.is_err() {
                            return;
                        }
                    }
                    None => {
                        output_open = false;
                    }
                }
            }
            result = &mut op => {
                break result;
            }
        }
    };

    while let Ok(chunk) = line_rx.try_recv() {
        if evt_tx
            .send(WorkerEvent::CustomActionOutput { chunk })
            .await
            .is_err()
        {
            return;
        }
    }

    let _distributions = wsl.list().await;
    let status_line = match result {
        Ok(()) => format!("Run {action_name} on {distro}"),
        Err(e) => friendly_status(&format!("Custom action {action_name} failed"), &e),
    };

    let _ = evt_tx
        .send(WorkerEvent::CustomActionFinished { status_line })
        .await;
}

