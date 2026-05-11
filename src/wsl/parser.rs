use crate::errors::WSLError;
use super::types::{Distro, DistroState};

pub fn parse_distros(decoded: &str) -> Result<Vec<Distro>, WSLError> {
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
