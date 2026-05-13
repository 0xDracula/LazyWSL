use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Distribution {
    pub id: Option<String>,
    pub name: String,
    pub state: DistroState,
    pub version: WslVersion,
    pub is_default: bool,
    pub install_path: Option<String>,
    pub size_byes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistroState {
    Running,
    Stopped,
    Installing,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WslVersion {
    V1,
    V2,
    Unknown(u8)
}

impl From<&str> for DistroState {
    fn from(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "running" => DistroState::Running,
            "stopped" => DistroState::Stopped,
            "installing" => DistroState::Installing,
            _ => DistroState::Unknown(s.to_string()),
        }
    }
}
impl fmt::Display for DistroState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DistroState::Stopped => write!(f, "Stopped"),
            DistroState::Running => write!(f, "Running"),
            DistroState::Installing => write!(f, "Installing"),
            DistroState::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for WslVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WslVersion::V1 => write!(f, "1"),
            WslVersion::V2 => write!(f, "2"),
            WslVersion::Unknown(n) => write!(f, "{}", n),
        }
    }
}

impl From<u8> for WslVersion {
    fn from(value: u8) -> Self {
        match value {
            1 => WslVersion::V1,
            2 => WslVersion::V2,
            other => WslVersion::Unknown(other),
        }
    }
}