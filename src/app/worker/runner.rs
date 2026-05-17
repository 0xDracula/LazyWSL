use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::{ self, MissedTickBehavior };
use crate::wsl::WSLService;
use crate::app::worker::commands::{ WorkerCmd, WorkerEvent };
use crate::config;

pub fn spawn_wsl_worker(
    cmd_rx: tokio::sync::mpsc::Receiver<WorkerCmd>,
    evt_tx: Sender<WorkerEvent>,
    wsl: Arc<dyn WSLService>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        run_wsl_worker(cmd_rx, evt_tx, wsl).await;
    })
}


async fn run_wsl_worker(
    mut cmd_rx: tokio::sync::mpsc::Receiver<WorkerCmd>,
    evt_tx: Sender<WorkerEvent>,
    wsl: Arc<dyn WSLService>,
) {
    let mut tick = time::interval_at(
        time::Instant::now() + Duration::from_secs(config::load_or_create().refresh_secs),
        Duration::from_secs(config::load_or_create().refresh_secs)
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
                if evt_tx.send(WorkerEvent::ListOnly { distributions: list }).await.is_err() {
                    break;
                }
            }
        }
    }
}

async fn process_cmd(wsl: &Arc<dyn WSLService>, evt_tx: &Sender<WorkerEvent>, cmd: WorkerCmd) {
    if let WorkerCmd::RunCustomAction { distro, action_name, command } = cmd {
        process_custom_action(wsl, evt_tx, distro, action_name, command).await;
        return;
    }
    let evt = match cmd {
        WorkerCmd::Refresh => {
            let distributions = wsl.list().await;
            let status_line = match &distributions {
                Ok(v) => format!("Loaded {} distro(s).", v.len()),
                Err(e) => format!("Refresh failed: {e}"),
            };
            WorkerEvent::DistroUpdated { distributions, status_line }
        }
        WorkerCmd::RunDistro(name) => {
            let op = wsl.run(&name).await;
            let distributions = wsl.list().await;
            let status_line = match op {
                Ok(()) => format!("Ran distro `{name}`."),
                Err(e) => format!("Run distro failed {e}."),
            };
            WorkerEvent::DistroUpdated { distributions, status_line }
        }
        WorkerCmd::Terminate(name) => {
            let op = wsl.terminate(&name).await;
            let distributions = wsl.list().await;
            let status_line = match op {
                Ok(()) => format!("Terminated `{name}`."),
                Err(e) => format!("Terminate Failed {e}"),
            };
            WorkerEvent::DistroUpdated { distributions, status_line }
        }
        WorkerCmd::Unregister(name) => {
            let op = wsl.unregister(&name).await;
            let distributions = wsl.list().await;
            let status_line = match op {
                Ok(()) => format!("Unregistered `{name}`."),
                Err(e) => format!("Failed to Unregister: {e}"),
            };
            WorkerEvent::DistroUpdated { distributions, status_line }
        }
        WorkerCmd::SetDefault(name) => {
            let op = wsl.set_default(&name).await;
            let distributions = wsl.list().await;
            let status_line = match op {
                Ok(()) => format!("`{name}` Set Default Successfully."),
                Err(e) => format!("Failed to Set Default: {e}"),
            };
            WorkerEvent::DistroUpdated { distributions, status_line }
        }
        WorkerCmd::Shutdown => {
            let op = wsl.shutdown().await;
            let distributions = wsl.list().await;
            let status_line = match op {
                Ok(()) => "Shutdown Succesful".to_string(),
                Err(e) => format!("Shutdown failed: {e}"),
            };
            WorkerEvent::DistroUpdated { distributions, status_line }
        }
        WorkerCmd::OpenShell(name) => {
            let op = wsl.open_shell(&name).await;
            let distributions = wsl.list().await;
            let status_line = match op {
                Ok(()) => format!("Opened shell for `{name}`"),
                Err(e) => format!("Open Shell Failed: {e}"),
            };
            WorkerEvent::DistroUpdated { distributions, status_line }
        }
        WorkerCmd::Import { name, tar_path, install_path } => {
            let op = wsl.import(&name, &tar_path, &install_path).await;
            let distributions = wsl.list().await;
            let status_line = match op {
                Ok(()) => format!("Imported `{name}`"),
                Err(e) => format!("Import failed {e}"),
            };
            WorkerEvent::DistroUpdated { distributions, status_line }
        }
        WorkerCmd::Export { distro, output } => {
            let op = wsl.export(&distro, &output).await;
            let distributions = wsl.list().await;
            let status_line = match op {
                Ok(()) => format!("Exported {distro}"),
                Err(e) => format!("Export Failed: {e}"),
            };
            WorkerEvent::DistroUpdated { distributions, status_line }
        }
        WorkerCmd::RunCustomAction { .. } => unreachable!(),
    };

    let _ = evt_tx.send(evt).await;
}

async fn process_custom_action(
    wsl: &Arc<dyn WSLService>,
    evt_tx: &Sender<WorkerEvent>,
    distro: String,
    action_name: String,
    command: String,
) {
    let (line_tx, mut line_rx) = tokio::sync::mpsc::channel::<String>(128);
    let op = wsl.run_custom_action(&distro, &command, line_tx);
    tokio::pin!(op);

    let result = loop {
        tokio::select! {
            line = line_rx.recv() => {
                if let Some(line) = line {
                    if evt_tx.send(WorkerEvent::CustomActionOutput { line }).await.is_err() {
                        return;
                    }
                }
            }
            result = &mut op => {
                break result;
            }
        }
    };

    while let Ok(line) = line_rx.try_recv() {
        if evt_tx
            .send(WorkerEvent::CustomActionOutput { line })
            .await
            .is_err() {
            return;
        }
    }

    let distributions = wsl.list().await;
    let status_line = match result {
        Ok(()) => format!("Run {action_name} on {distro}"),
        Err(e) => format!("Custom action {action_name} failed: {e}")
    };

    let _ = evt_tx
        .send(WorkerEvent::CustomActionFinished { distributions, status_line }).await;
}