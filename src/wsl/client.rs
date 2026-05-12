use std::process::Output;
use tokio::process::Command;
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
    async fn run_wsl(&self, args: &[&str]) -> Result<Output, WSLError> {
        let output = Command::new("wsl.exe")
            .args(args)
            .output()
            .await
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

    pub async fn get_distros(&self) -> Result<Vec<Distribution>, WSLError> {
        let output = self.run_wsl(&["--list", "--verbose"])
            .await?;

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

    pub async fn terminate(&self, name: &str) -> Result<(), WSLError> {
        let distros = self.get_distros().await?;
        distro_exists(name, &distros)?;

        self.run_wsl(&["--terminate", name]).await?;
        Ok(())
    }

    pub async fn unregister(&self, name: &str) -> Result<(), WSLError> {
        let distros = self.get_distros().await?;
        distro_exists(name, &distros)?;

        self.run_wsl(&["--unregister", name]).await?;

        Ok(())
    }

    pub async fn set_default(&self, name: &str) -> Result<(), WSLError> {
        let distros = self.get_distros().await?;
        distro_exists(name, &distros)?;

        self.run_wsl(&["--set-default", name]).await?;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), WSLError> {
        self.run_wsl(&["--shutdown"]).await?;
        Ok(())
    }

    pub async fn open_shell(&self, name: &str) -> Result<(), WSLError> {
        let distros = self.get_distros().await?;
        distro_exists(name, &distros)?;

        Command::new("wt.exe")
            .args(["wsl.exe", "-d", name])
            .spawn()?;

        Ok(())
    }

    pub async fn run_distro(&self, name: &str) -> Result<(), WSLError> {
        let distros = self.get_distros().await?;
        distro_exists(name, &distros)?;
        self.run_wsl(&["--distribution", name]).await?;

        Ok(())
    }

}

