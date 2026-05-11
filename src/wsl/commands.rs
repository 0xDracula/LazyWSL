use std::process::Command;
use super::types::{ Distro };
use super::parser::{ parse_distros };


// helpers

pub fn distro_exists(name: &str) -> bool {
    match get_distros() {
        Ok(distros) => distros.iter().any(|d| d.name == name),
        Err(_) => false
    }
}

// reads

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


//actions

pub fn terminate(name: &str) -> anyhow::Result<()> {
    if !distro_exists(name) {
        return Err(anyhow::anyhow!("Not found!"));
    }

    Command::new("wsl.exe")
        .args(["--terminate", name])
        .output()
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}

pub fn unregister(name: &str) -> anyhow::Result<()> {
    if !distro_exists(name) {
        return Err(anyhow::anyhow!("Distro {} not found!", name));
    }

    Command::new("wsl.exe")
        .args(["--unregister", name])
        .output()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}

pub fn set_default(name: &str) -> anyhow::Result<()> {
    Command::new("wsl.exe")
        .args(["--set-default", name])
        .output()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}

pub fn shutdown() -> anyhow::Result<()> {
    Command::new("wsl.exe")
        .args(["--shutdown"])
        .output()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}

pub fn open_shell(name: &str) -> anyhow::Result<()> {
    if !distro_exists(name) {
        return Err(anyhow::anyhow!("Distro {} not found", name));
    }
    Command::new("wsl.exe")
        .args(["-d", name])
        .output()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}

