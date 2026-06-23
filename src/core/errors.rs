use thiserror::Error;

#[derive(Error, Debug)]
pub enum WSLError {
    #[error("WSL is not installed on this system")]
    NotInstalled,

    #[error("No WSL distros found")]
    NoDistros,

    #[error("Distro {0} not found!")]
    DistroNotFound(String),

    #[error("Failed to run WSL command: {0}")]
    CommandFailed(#[from] std::io::Error),

    #[error("WSL process failed with exit code {code}: {stderr}")]
    ProcessFailed { code: i32, stderr: String },
}
