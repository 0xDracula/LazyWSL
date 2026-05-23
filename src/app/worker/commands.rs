use crate::core::{Distribution, WSLError};
use std::path::PathBuf;
use tokio::sync::mpsc::Receiver;

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
    RunCustomAction {
        distro: String,
        action_name: String,
        command: String,
        input_rx: Receiver<String>,
    },
    Batch(Vec<WorkerCmd>),
}

#[derive(Debug)]
pub enum WorkerEvent {
    StateRefresh {
        distributions: Result<Vec<Distribution>, WSLError>,
        status_line: Option<String>,
    },
    CustomActionOutput {
        chunk: String,
    },
    CustomActionFinished {
        status_line: String,
    },
}
