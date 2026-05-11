use std::process::Command;
use super::types::{ Distro };
use super::parser::{ parse_distros };
pub fn get_distros() -> anyhow::Result<Vec<Distro>> {
    let output = Command::new("wsl.exe")
        .args(["--list", "--verbose"])
        .output()
        .map_err(|_| anyhow::anyhow!("WSL is not installed!"))?;

    if output.stdout.is_empty() {
        return Err(anyhow::anyhow!("No distros found!"));
    }
    // Convert UTF-16 to UTF-8 due to wsl output format
    let utf16: Vec<u16> = output.stdout
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();

    let decoded = String::from_utf16_lossy(&utf16);
    parse_distros(&decoded)
}