use std::path::PathBuf;
use crate::core::{ Distribution, WSLError };

pub enum WorkerCmd {
    Refresh,
    RunDistro(String),
    Terminate(String),
    Unregister(String),
    SetDefault(String),
    Shutdown,
    OpenShell(String),
    Export { distro: String, output: PathBuf },
    Import { name: String, tar_path: PathBuf, install_path: PathBuf },
    RunCustomAction { distro: String, action_name: String, command: String },
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
    CustomActionOutput { line:String },
    CustomActionFinished { distributions: Result<Vec<Distribution>, WSLError>, status_line: String },
}