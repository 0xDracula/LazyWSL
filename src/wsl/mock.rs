use crate::core::{Distribution, DistroState, WSLError, WslVersion};
use crate::wsl::{CatalogEntry, WSLService};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct MockWSLService {
    distros: Mutex<Vec<Distribution>>,
}

impl MockWSLService {
    pub fn new() -> Self {
        Self {
            distros: Mutex::new(sample_distros()),
        }
    }

    fn ensure_exists(&self, name: &str) -> Result<(), WSLError> {
        if self.distros.lock().unwrap().iter().any(|d| d.name == name) {
            Ok(())
        } else {
            Err(WSLError::DistroNotFound(name.to_string()))
        }
    }
}

impl Default for MockWSLService {
    fn default() -> Self {
        Self::new()
    }
}

fn sample_distros() -> Vec<Distribution> {
    vec![
        Distribution {
            id: Some("{00000000-0000-0000-0000-000000000001}".to_string()),
            name: "Ubuntu".to_string(),
            state: DistroState::Running,
            version: WslVersion::V2,
            is_default: true,
            install_path: Some(
                r"C:\Users\dev\AppData\Local\Packages\Ubuntu\LocalState".to_string(),
            ),
            size_bytes: Some(8_589_934_592),
        },
        Distribution {
            id: Some("{00000000-0000-0000-0000-000000000002}".to_string()),
            name: "Debian".to_string(),
            state: DistroState::Stopped,
            version: WslVersion::V2,
            is_default: false,
            install_path: Some(
                r"C:\Users\dev\AppData\Local\Packages\Debian\LocalState".to_string(),
            ),
            size_bytes: Some(2_147_483_648),
        },
        Distribution {
            id: Some("{00000000-0000-0000-0000-000000000003}".to_string()),
            name: "docker-desktop".to_string(),
            state: DistroState::Stopped,
            version: WslVersion::V1,
            is_default: false,
            install_path: None,
            size_bytes: None,
        },
    ]
}

#[async_trait]
impl WSLService for MockWSLService {
    async fn list(&self) -> Result<Vec<Distribution>, WSLError> {
        tokio::time::sleep(Duration::from_millis(120)).await;
        Ok(self.distros.lock().unwrap().clone())
    }

    async fn run(&self, name: &str) -> Result<(), WSLError> {
        self.ensure_exists(name)?;
        if let Some(d) = self
            .distros
            .lock()
            .unwrap()
            .iter_mut()
            .find(|d| d.name == name)
        {
            d.state = DistroState::Running
        }
        Ok(())
    }

    async fn terminate(&self, name: &str) -> Result<(), WSLError> {
        self.ensure_exists(name)?;
        if let Some(d) = self
            .distros
            .lock()
            .unwrap()
            .iter_mut()
            .find(|d| d.name == name)
        {
            d.state = DistroState::Stopped;
        }

        Ok(())
    }

    async fn unregister(&self, name: &str) -> Result<(), WSLError> {
        self.ensure_exists(name)?;
        self.distros.lock().unwrap().retain(|d| d.name != name);
        Ok(())
    }

    async fn set_default(&self, name: &str) -> Result<(), WSLError> {
        self.ensure_exists(name)?;
        for d in self.distros.lock().unwrap().iter_mut() {
            d.is_default = d.name == name;
        }
        Ok(())
    }

    async fn open_shell(&self, name: &str) -> Result<(), WSLError> {
        self.ensure_exists(name)?;
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), WSLError> {
        for d in self.distros.lock().unwrap().iter_mut() {
            d.state = DistroState::Stopped;
        }
        Ok(())
    }

    async fn import(
        &self,
        name: &str,
        _tar_path: &Path,
        install_path: &Path,
    ) -> Result<(), WSLError> {
        let mut distros = self.distros.lock().unwrap();

        distros.retain(|d| d.name != name);
        distros.push(Distribution {
            id: None,
            name: name.to_string(),
            state: DistroState::Stopped,
            version: WslVersion::V2,
            is_default: false,
            install_path: Some(install_path.to_string_lossy().to_string()),
            size_bytes: Some(1_073_741_824),
        });
        Ok(())
    }

    async fn export(&self, distro: &str, output: &Path) -> Result<(), WSLError> {
        self.ensure_exists(distro)?;
        if let Some(parent) = output.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        std::fs::write(output, "mock for snapshot distro\n").map_err(|e| {
            WSLError::ProcessFailed {
                code: 1,
                stderr: e.to_string(),
            }
        })?;
        Ok(())
    }

    async fn run_custom_action(
        &self,
        distro: &str,
        command: &str,
        output_tx: Sender<String>,
        mut input_rx: Receiver<String>,
    ) -> Result<(), WSLError> {
        self.ensure_exists(distro)?;

        let _ = output_tx
            .send(format!("[mock] running on {distro}: {command}\n"))
            .await;

        for i in 1..=3 {
            tokio::time::sleep(Duration::from_millis(250)).await;
            if output_tx
                .send(format!("[mock] line {i} of output\n"))
                .await
                .is_err()
            {
                break;
            }
        }

        while let Ok(line) = input_rx.try_recv() {
            let _ = output_tx
                .send(format!("[mock] received input: {line}"))
                .await;
        }

        let _ = output_tx.send("[mock] done\n".to_string()).await;

        Ok(())
    }

    async fn list_online(&self) -> Result<Vec<CatalogEntry>, WSLError> {
        tokio::time::sleep(Duration::from_millis(150)).await;
        Ok(vec![
            CatalogEntry {
                name: "Ubuntu".into(),
                friendly: "Ubuntu".into(),
            },
            CatalogEntry {
                name: "Ubuntu-22.04".into(),
                friendly: "Ubuntu 22.04 LTS".into(),
            },
            CatalogEntry {
                name: "Ubuntu-24.04".into(),
                friendly: "Ubuntu 24.04 LTS".into(),
            },
            CatalogEntry {
                name: "Debian".into(),
                friendly: "Debian GNU/Linux".into(),
            },
            CatalogEntry {
                name: "kali-linux".into(),
                friendly: "Kali Linux Rolling".into(),
            },
            CatalogEntry {
                name: "openSUSE-Tubmleweed".into(),
                friendly: "openSUSE Tubmleweed".into(),
            },
        ])
    }

    async fn install_streaming(
        &self,
        name: &str,
        output_tx: Sender<String>,
    ) -> Result<(), WSLError> {
        let _ = output_tx
            .send(format!("$ wsl --install -d {name} --no-launch\n"))
            .await;

        for pct in [10, 35, 60, 85, 100] {
            tokio::time::sleep(Duration::from_millis(300)).await;
            if output_tx
                .send(format!("DOwnloading: {name} {pct}%\n"))
                .await
                .is_err()
            {
                break;
            }
        }

        let _ = output_tx.send("Installed.\n".to_string()).await;
        self.distros.lock().unwrap().push(Distribution {
            id: None,
            name: name.to_string(),
            state: DistroState::Stopped,
            version: WslVersion::V2,
            is_default: false,
            install_path: None,
            size_bytes: Some(1_073_741_824),
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_then_terminate() {
        let svc = MockWSLService::new();
        svc.run("Debian").await.unwrap();
        let running = svc.list().await.unwrap();
        assert_eq!(
            running.iter().find(|d| d.name == "Debian").unwrap().state,
            DistroState::Running
        );

        svc.terminate("Debian").await.unwrap();
        let stopped = svc.list().await.unwrap();
        assert_eq!(
            stopped.iter().find(|d| d.name == "Debian").unwrap().state,
            DistroState::Stopped
        );
    }

    #[tokio::test]
    async fn set_default_is_exclusive() {
        let svc = MockWSLService::new();
        svc.set_default("Debian").await.unwrap();
        let list = svc.list().await.unwrap();
        let defaults: Vec<_> = list
            .iter()
            .filter(|d| d.is_default)
            .map(|d| &d.name)
            .collect();
        assert_eq!(defaults, vec!["Debian"]);
    }

    #[tokio::test]
    async fn unregsiter_removes_and_missing_errors() {
        let svc = MockWSLService::new();
        svc.unregister("docker-desktop").await.unwrap();
        assert!(
            svc.list()
                .await
                .unwrap()
                .iter()
                .all(|d| d.name != "docker-desktop")
        );
        assert!(matches!(
            svc.terminate("docker-desktop").await,
            Err(WSLError::DistroNotFound(_))
        ))
    }
}
