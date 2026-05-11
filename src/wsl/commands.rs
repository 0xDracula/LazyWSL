use std::process::Command;
use super::types::{ Distro };
use super::parser::{ parse_distros };
use crate::errors::*;
// helpers

pub fn distro_exists(name: &str) -> bool {
    match get_distros() {
        Ok(distros) => distros.iter().any(|d| d.name == name),
        Err(_) => false
    }
}

// reads

pub fn get_distros() -> Result<Vec<Distro>, WSLError> {
    let output = Command::new("wsl.exe")
        .args(["--list", "--verbose"])
        .output()
        .map_err(|_| WSLError::NotInstalled)?;

    if output.stdout.is_empty() {
        return Err(WSLError::NoDistros);
    }
    // Convert UTF-16 to UTF-8 due to wsl output format
    let utf16: Vec<u16> = output.stdout
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();

    let decoded = String::from_utf16_lossy(&utf16);
    parse_distros(&decoded)
}


//actions

pub fn terminate(name: &str) -> Result<(), WSLError> {
    if !distro_exists(name) {
        return Err(WSLError::DistroNotFound(name.to_string()));
    }

    Command::new("wsl.exe")
        .args(["--terminate", name])
        .output()?;
    Ok(())
}

pub fn unregister(name: &str) -> Result<(), WSLError> {
    if !distro_exists(name) {
        return Err(WSLError::DistroNotFound(name.to_string()));
    }

    Command::new("wsl.exe")
        .args(["--unregister", name])
        .output()?;

    Ok(())
}

pub fn set_default(name: &str) -> Result<(), WSLError> {
    if !distro_exists(name) {
        return Err(WSLError::DistroNotFound(name.to_string()));
    }
    Command::new("wsl.exe")
        .args(["--set-default", name])
        .output()?;

    Ok(())
}

pub fn shutdown() -> Result<(), WSLError> {
    Command::new("wsl.exe")
        .args(["--shutdown"])
        .output()?;
    Ok(())
}

pub fn open_shell(name: &str) -> Result<(), WSLError> {
    if !distro_exists(name) {
        return Err(WSLError::DistroNotFound(name.to_string()));
    }
    Command::new("wt.exe")
        .args(["wsl.exe", "-d", name])
        .spawn()?;

    Ok(())
}

