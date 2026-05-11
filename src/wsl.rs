use std::fmt;

pub struct Distro {
    pub name: String,
    pub state: DistroState,
    pub version: u8,
    pub is_default: bool,
}

pub enum DistroState {
    RUNNING,
    STOPPED,
}

impl fmt::Display for DistroState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DistroState::STOPPED => write!(f, "Stopped"),
            DistroState::RUNNING => write!(f, "Running"),
        }
    }
}
