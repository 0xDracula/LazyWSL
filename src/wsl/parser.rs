use crate::core::{Distribution, DistroState};
use crate::wsl::WslVersion;
use std::fs;
use std::path::Path;

#[cfg(windows)]
use winreg::RegKey;

#[cfg(windows)]
use winreg::enums::HKEY_CURRENT_USER;

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

    let install_paths = get_distro_path().unwrap_or_default();
    for distro in &mut distros {
        distro.install_path = install_paths
            .iter()
            .find(|(name, _)| name == &distro.name)
            .map(|(_, path)| path.clone());
    }

    for distro in &mut distros {
        if let Some(path) = distro.install_path.as_ref() {
            distro.size_bytes = get_distro_size(path);
        }
    }

    distros
}

#[cfg(not(windows))]
pub fn get_distro_path() -> std::io::Result<Vec<(String, String)>> {
    Ok(vec![])
}

#[cfg(windows)]
pub fn get_distro_path() -> std::io::Result<Vec<(String, String)>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let lxss = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Lxss")?;

    let mut result = Vec::new();
    for guid in lxss.enum_keys() {
        let guid = guid?;
        let key = lxss.open_subkey(&guid)?;

        let name: String = match key.get_value("DistributionName") {
            Ok(v) => v,
            Err(_) => continue,
        };
        let install_path: String = match key.get_value("BasePath") {
            Ok(v) => v,
            Err(_) => continue,
        };
        let install_path = install_path
            .strip_prefix(r"\\?\")
            .unwrap_or(&install_path)
            .to_string();

        result.push((name, install_path));
    }

    Ok(result)
}

pub fn get_distro_size(install_path: &String) -> Option<u64> {
    let vhdx = Path::new(install_path).join("ext4.vhdx");
    let metadata = fs::metadata(vhdx).ok()?;
    Some(metadata.len())
}

pub fn parse_line_distro(line: &str) -> Option<Distribution> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let is_default = line.starts_with("*") || line.starts_with(" *");
    let normalized = line.replace("*", " ");
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
        install_path: None,
        size_bytes: None,
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

        let docker = list
            .iter()
            .find(|d| d.name == "docker-desktop")
            .expect("dd");
        assert!(!docker.is_default);
        assert_eq!(docker.state, DistroState::Stopped);
        assert_eq!(docker.version, WslVersion::V1);

        let test_distro = list.iter().find(|d| d.name == "Test Distro").expect("td");
        assert_eq!(test_distro.name, "Test Distro");
    }
}
