use std::process::{Command, Output};
use super::types::{ Distribution };
use super::parser::{ parse_wsl_output };
use crate::errors::*;
pub fn distro_exists(name: &str, distros: &[Distribution]) -> Result<(), WSLError> {
    if distros.iter().any(|d| d.name == name) {
        Ok(())
    } else {
        Err(WSLError::DistroNotFound(name.to_string()))
    }
}

pub struct WslProcess;

impl WslProcess {

    // helpers

    pub fn new() -> Self {
        Self
    }
    pub fn run_wsl(&self, args: &[&str]) -> Result<Output, WSLError> {
        let output = Command::new("wsl.exe")
            .args(args)
            .output()
            .map_err(|_| WSLError::NotInstalled)?;

        if !output.status.success() {
            return Err(WSLError::CommandFailed(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("wsl.exe exited with {}", output.status),
            )));
        }

        Ok(output)
    }

    // reads

    pub fn get_distros(&self) -> Result<Vec<Distribution>, WSLError> {
        let output = self.run_wsl(&["--list", "--verbose"])?;

        if output.stdout.is_empty() {
            return Err(WSLError::NoDistros);
        }

        // Convert UTF-16 to UTF-8 due to wsl output format
        let utf16: Vec<u16> = output.stdout
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();

        let decoded = String::from_utf16_lossy(&utf16);
        Ok(parse_wsl_output(&decoded))
    }


    //actions

    pub fn terminate(&self, name: &str) -> Result<(), WSLError> {
        let distros = self.get_distros()?;
        distro_exists(name, &distros)?;
        self.run_wsl(&["--terminate", name])?;
        Ok(())
    }

    pub fn unregister(&self, name: &str) -> Result<(), WSLError> {
        let distros = self.get_distros()?;
        distro_exists(name, &distros)?;

        self.run_wsl(&["--unregister", name])?;

        Ok(())
    }

    pub fn set_default(&self, name: &str) -> Result<(), WSLError> {
        let distros = self.get_distros()?;
        distro_exists(name, &distros)?;

        self.run_wsl(&["--set-default", name])?;

        Ok(())
    }

    pub fn shutdown(&self) -> Result<(), WSLError> {
        self.run_wsl(&["--shutdown"])?;
        Ok(())
    }

    pub fn open_shell(&self, name: &str) -> Result<(), WSLError> {
        let distros = self.get_distros()?;
        distro_exists(name, &distros)?;

        Command::new("wt.exe")
            .args(["wsl.exe", "-d", name])
            .spawn()?;

        Ok(())
    }

}

