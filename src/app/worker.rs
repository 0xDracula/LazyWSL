use std::path::PathBuf;
use std::time::{Duration};
use tokio::sync::mpsc::Sender;
use tokio::time;
use tokio::time::MissedTickBehavior;
use crate::errors::WSLError;
use crate::wsl::{Distribution, WslProcess};

pub enum WorkerCmd {
    Refresh,
    RunDistro(String),
    Terminate(String),
    Unregister(String),
    SetDefault(String),
    Shutdown,
    OpenShell(String),
    Export {
        distro: String,
        output: PathBuf,
    },
    Import {
        name: String,
        tar_path: PathBuf,
        install_path: PathBuf,
    },
}

#[derive(Debug)]
pub enum WorkerEvent {
    DistroUpdated {
        distributions: Result<Vec<Distribution>, WSLError>,
        status_line: String,
    },
    ListOnly {
        distributions: Result<Vec<Distribution>, WSLError>,
    },
}

pub fn spawn_wsl_worker(
    cmd_rx: tokio::sync::mpsc::Receiver<WorkerCmd>,
    evt_tx: Sender<WorkerEvent>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        run_wsl_worker(cmd_rx, evt_tx).await;
    })
}

async fn run_wsl_worker(
    mut cmd_rx: tokio::sync::mpsc::Receiver<WorkerCmd>,
    evt_tx: Sender<WorkerEvent>,
) {
    let wsl = WslProcess::new();

    let mut tick = time::interval_at(
        time::Instant::now() + Duration::from_secs(2),
        Duration::from_secs(2),
    );
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            biased;

            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(cmd) => process_cmd(&wsl, &evt_tx, cmd).await,
                    None => break,
                }
            }
            _ = tick.tick() => {
                let list = wsl.get_distros().await;
                if evt_tx.send(WorkerEvent::ListOnly { distributions: list }).await.is_err() {
                    break;
                }
            }
        }
    }
}

async fn process_cmd(wsl: &WslProcess, evt_tx: &Sender<WorkerEvent>, cmd: WorkerCmd) {
    let evt = match cmd {
        WorkerCmd::Refresh => {
            let distributions = wsl.get_distros().await;
            let status_line = match &distributions {
                Ok(v) => format!("Loaded {} distro(s).", v.len()),
                Err(e) => format!("Refresh failed: {e}"),
            };
            WorkerEvent::DistroUpdated {
                distributions,
                status_line,
            }
        }
        WorkerCmd::RunDistro(name) => {
            let op = wsl.run_distro(&name).await;
            let distributions = wsl.get_distros().await;
            let status_line = match op {
                Ok(()) => format!("Ran distro `{name}`."),
                Err(e) => format!("Run distro failed {e}."),
            };
            WorkerEvent::DistroUpdated {
                distributions,
                status_line,
            }
        }
        WorkerCmd::Terminate(name) => {
            let op = wsl.terminate(&name).await;
            let distributions = wsl.get_distros().await;
            let status_line = match op {
                Ok(()) => format!("Terminated `{name}`."),
                Err(e) => format!("Terminate Failed: {e}"),
            };
            WorkerEvent::DistroUpdated {
                distributions,
                status_line,
            }
        }
        WorkerCmd::Unregister(name) => {
            let op = wsl.unregister(&name).await;
            let distributions = wsl.get_distros().await;
            let status_line = match op {
                Ok(()) => format!("Unregistered `{name}`."),
                Err(e) => format!("Failed to unregister: {e}"),
            };
            WorkerEvent::DistroUpdated {
                distributions,
                status_line,
            }
        }
        WorkerCmd::SetDefault(name) => {
            let op = wsl.set_default(&name).await;
            let distributions = wsl.get_distros().await;
            let status_line = match op {
                Ok(()) => format!("`{name}` Set Default Successfully."),
                Err(e) => format!("Failed to Set Default: {e}"),
            };
            WorkerEvent::DistroUpdated {
                distributions,
                status_line,
            }
        }
        WorkerCmd::Shutdown => {
            let op = wsl.shutdown().await;
            let distributions = wsl.get_distros().await;
            let status_line = match op {
                Ok(()) => "Shutdown Successful".to_string(),
                Err(e) => format!("Shutdown Failed: {e}"),
            };
            WorkerEvent::DistroUpdated {
                distributions,
                status_line,
            }
        }
        WorkerCmd::OpenShell(name) => {
            let op = wsl.open_shell(&name).await;
            let distributions = wsl.get_distros().await;
            let status_line = match op {
                Ok(()) => format!("Opened Shell for `{name}`."),
                Err(e) => format!("Open Shell Failed: {e}"),
            };
            WorkerEvent::DistroUpdated {
                distributions,
                status_line,
            }
        }
        WorkerCmd::Import { name, tar_path, install_path } => {
            let op = wsl.import(&name, &tar_path, &install_path).await;

            let distributions = wsl.get_distros().await;

            let status_line = match op {
                Ok(()) => format!("Imported `{name}`"),
                Err(e) => format!("Import failed {e}"),
            };

            WorkerEvent::DistroUpdated {
                distributions,
                status_line
            }
        }
        WorkerCmd::Export { distro, output } => {
            let op = wsl.export(&distro, &output).await;

            let distributions = wsl.get_distros().await;

            let status_line = match op {
                Ok(()) => format!("Exported {distro}"),
                Err(e) => format!("Export failed {e}"),
            };

            WorkerEvent::DistroUpdated {
                distributions,
                status_line
            }
        }
    };

    let _ = evt_tx.send(evt).await;
}