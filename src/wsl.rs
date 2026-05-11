use std::fmt;
use std::process::Command;

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

fn parse_distros(decoded: &str) -> anyhow::Result<Vec<Distro>> {
    let mut distros = vec![];

    for line in decoded.lines().skip(1) {
        let line = line.trim();

        if line.is_empty() {continue};
        let is_default = line.starts_with("*");
        let line = line.trim_start_matches("*").trim();

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {continue};

        distros.push(Distro {
            name: parts[0].to_string(),
            state: match parts[1] {
                "Running" => DistroState::RUNNING,
                _ => DistroState::STOPPED,
            },
            version: parts[2].parse().unwrap_or(2),
            is_default,
        })
    }

    Ok(distros)
}

pub fn get_distros() -> anyhow::Result<Vec<Distro>> {
    let output = Command::new("wsl.exe")
        .args(["--list", "--verbose"])
        .output()?;

    // Convert UTF-16 to UTF-8 due to wsl output format
    let utf16: Vec<u16> = output.stdout
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();

    let decoded = String::from_utf16_lossy(&utf16);
    parse_distros(&decoded)
}