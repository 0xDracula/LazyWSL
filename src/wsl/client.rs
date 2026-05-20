use std::path::Path;
use std::process::{Output, Stdio};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::timeout;
use crate::config;
use super::parser::parse_wsl_output;
use crate::core::{ Distribution, WSLError };

pub fn distro_exists(name: &str, distros: &[Distribution]) -> Result<(), WSLError> {
    if distros.iter().any(|d| d.name == name) {
        Ok(())
    } else {
        Err(WSLError::DistroNotFound(name.to_string()))
    }
}

pub struct WslProcess;

impl WslProcess {
    pub fn new() -> Self { Self }

    async fn run_wsl(&self, args: &[&str]) -> Result<Output, WSLError> {
        self.run_wsl_with_timeout(args, config::load_or_create().timeouts.default_secs).await
    }
    async fn run_wsl_with_timeout(&self, args: &[&str], timeout_secs: u64) -> Result<Output, WSLError> {
        let output =  timeout(Duration::from_secs(timeout_secs), async {
            Command::new("wsl.exe")
                .args(args)
                .output()
                .await
        }).await.map_err(|_| WSLError::CommandFailed(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            format!("WSL command timed out after {timeout_secs}s"),
        )))?.map_err(|_| WSLError::NotInstalled)?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            let stderr_bytes = if output.stderr.is_empty() {
                &output.stdout
            } else {
                &output.stderr
            };

            let utf16: Vec<u16> = stderr_bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();

            let mut stderr = String::from_utf16_lossy(&utf16).trim().to_string();
            if stderr.is_empty() || stderr.contains('\u{FFFD}') {
                stderr = String::from_utf8_lossy(stderr_bytes).replace('\0', "").trim().to_string();
            }

            return Err(WSLError::ProcessFailed {
                code,
                stderr
            })
        }

        Ok(output)
    }

    async fn run_wsl_quick(&self, args: &[&str]) -> Result<Output, WSLError> {
        self.run_wsl_with_timeout(args, config::load_or_create().timeouts.quick_secs).await
    }

    async fn run_wsl_long(&self, args: &[&str]) -> Result<Output, WSLError> {
        self.run_wsl_with_timeout(args, config::load_or_create().timeouts.long_secs).await
    }

    pub async fn get_distros(&self) -> Result<Vec<Distribution>, WSLError> {
        let output = self.run_wsl(&["--list", "--verbose"]).await?;

        if output.stdout.is_empty() { return Err(WSLError::NoDistros); }

        // Convert UTF-16 to UTF-8 due to wsl output format
        let utf16: Vec<u16> = output.stdout
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();

        let decoded = String::from_utf16_lossy(&utf16);
        Ok(parse_wsl_output(&decoded))
    }

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
        self.run_wsl_quick(&["--set-default", name]).await?;
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
        self.run_wsl(&["--distribution", name, "--", "/bin/true"]).await?;
        Ok(())
    }

    pub async fn import(&self, name: &str, tar_path: &Path, install_path: &Path) -> Result<(), WSLError> {
        self.run_wsl_long(&["--import", name, &install_path.to_string_lossy(), &tar_path.to_string_lossy()]).await?;
        Ok(())
    }

    pub async fn export(&self, distro: &str, output: &Path) -> Result<(), WSLError> {
        self.run_wsl_long(&["--export", distro, &output.to_string_lossy()]).await?;
        Ok(())
    }

    pub async fn run_custom_action(&self, distro: &str, command: &str, output_tx: Sender<String>, mut input_rx: Receiver<String>) -> Result<(), WSLError> {
        let distros = self.get_distros().await?;
        distro_exists(distro, &distros)?;

        let mut child = Command::new("wsl.exe")
            .args(["--distribution", distro, "--", "script", "-qfec", command, "/dev/null"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let stdin_task = stdin.map(|mut stdin| {
            tokio::spawn(async move {
                while let Some(input) = input_rx.recv().await {
                    stdin.write_all(input.as_bytes()).await?;
                    stdin.flush().await?;
                }
                Ok::<(), std::io::Error>(())
            })
        });

        let stdout_task = stdout.map(|stdout| stream_output(stdout, output_tx.clone(), None));
        let stderr_task = stderr.map(|stderr| stream_output(stderr, output_tx, Some("stderr: ")));

        let status = child.wait().await?;

        if let Some(task) = stdout_task {
            let _ = task.await;
        }

        if let Some(task) = stderr_task {
            let _ = task.await;
        }

        if let Some(task) = stdin_task {
            task.abort();
        }

        if !status.success() {
            return Err(WSLError::CommandFailed(std::io::Error::other(format!("Custom action exited with {status}"))));
        }
        Ok(())
    }
}

fn stream_output<R> (
    mut reader: R,
    tx: Sender<String>,
    prefix: Option<&'static str>,
) -> tokio::task::JoinHandle<Result<(), std::io::Error>>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut buffer = [0_u8; 1024];
        loop {
            let n = reader.read(&mut buffer).await?;
            if n == 0 {
                break;
            }

            let chunk = String::from_utf8_lossy(&buffer[..n]).to_string();
            let chunk = match prefix {
                Some(prefix) => format!("{prefix}{chunk}"),
                None => chunk
            };

            if tx.send(chunk).await.is_err() {
                break;
            }
        }
        Ok(())
    })
}