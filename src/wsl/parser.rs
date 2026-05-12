use crate::errors::WSLError;
use crate::wsl::WslVersion;
use super::types::{Distribution, DistroState};

pub fn parse_wsl_output(decoded: &str) -> Vec<Distribution> {
    let mut distros = vec![];

    if decoded.trim().is_empty() {
        return distros;
    }

    let lines: Vec<&str> = decoded.lines().collect();

    for line in lines.iter().skip(1) {
        if let Some(distro) = parse_line_distro(line) {
            distros.push(distro);
        }
    }

    distros
}

pub fn parse_line_distro(line: &str) -> Option<Distribution> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    // check if it's the default distro
    let is_default = line.starts_with("*") || line.starts_with(" *");

    // remove the asterisk if present, and normalize spacing
    let normalized = line.replace("*", " ");

    // split by whitespaces and filter empty strings
    let parts: Vec<&str> = normalized.split_whitespace().collect();

    if parts.len() < 3 {
        return None;
    }

    let version_str = parts.last()?;
    let version_u8: u8 = version_str.parse().ok()?;
    let version = WslVersion::from(version_u8);
    
    let state_str = parts.get(parts.len() - 2)?;
    let state = DistroState::from(*state_str);

    let name_parts = &parts[..parts.len() - 2];
    let name = name_parts.join(" ");

    Some(Distribution {
        id: None,
        name,
        state,
        version,
        is_default,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = concat!(
        "  NAME             STATE           VERSION\n",
        "* Ubuntu           Running         2\n",
        "  docker-desktop   Stopped         1\n",
        "  Test Distro      Stopped         2\n",
    );

    #[test]
    fn parse_default_star_and_multiword_name() {
        let list = parse_wsl_output(SAMPLE);
        assert_eq!(list.len(), 3);

        let ubuntu = list.iter().find(|d| d.name == "Ubuntu").expect("ubuntu");
        assert!(ubuntu.is_default);
        assert_eq!(ubuntu.state, DistroState::Running);
        assert_eq!(ubuntu.version, WslVersion::V2);

        let docker = list.iter().find(|d| d.name == "docker-desktop").expect("dd");
        assert!(!docker.is_default);
        assert_eq!(docker.state, DistroState::Stopped);
        assert_eq!(docker.version, WslVersion::V1);

        let test_distro = list.iter().find(|d| d.name == "Test Distro").expect("td");
        assert_eq!(test_distro.name, "Test Distro");
    }
}