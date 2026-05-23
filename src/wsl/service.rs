use crate::core::{Distribution, WSLError};
use async_trait::async_trait;
use std::path::Path;
use tokio::sync::mpsc::{Receiver, Sender};

#[async_trait]
pub trait WSLService: Send + Sync {
    async fn list(&self) -> Result<Vec<Distribution>, WSLError>;
    async fn run(&self, name: &str) -> Result<(), WSLError>;
    async fn terminate(&self, name: &str) -> Result<(), WSLError>;
    async fn unregister(&self, name: &str) -> Result<(), WSLError>;
    async fn set_default(&self, name: &str) -> Result<(), WSLError>;
    async fn open_shell(&self, name: &str) -> Result<(), WSLError>;
    async fn shutdown(&self) -> Result<(), WSLError>;
    async fn import(
        &self,
        name: &str,
        tar_path: &std::path::Path,
        install_path: &std::path::Path,
    ) -> Result<(), WSLError>;
    async fn export(&self, distro: &str, output: &std::path::Path) -> Result<(), WSLError>;
    async fn run_custom_action(
        &self,
        distro: &str,
        command: &str,
        output_tx: Sender<String>,
        input_rx: Receiver<String>,
    ) -> Result<(), WSLError>;
}

pub struct WSLProcessService {
    inner: super::client::WslProcess,
}

impl WSLProcessService {
    pub fn new() -> Self {
        Self {
            inner: super::client::WslProcess::new(),
        }
    }
}

#[async_trait]
impl WSLService for WSLProcessService {
    async fn list(&self) -> Result<Vec<Distribution>, WSLError> {
        self.inner.get_distros().await
    }

    async fn run(&self, name: &str) -> Result<(), WSLError> {
        self.inner.run_distro(name).await
    }

    async fn terminate(&self, name: &str) -> Result<(), WSLError> {
        self.inner.terminate(name).await
    }

    async fn unregister(&self, name: &str) -> Result<(), WSLError> {
        self.inner.unregister(name).await
    }

    async fn set_default(&self, name: &str) -> Result<(), WSLError> {
        self.inner.set_default(name).await
    }

    async fn open_shell(&self, name: &str) -> Result<(), WSLError> {
        self.inner.open_shell(name).await
    }

    async fn shutdown(&self) -> Result<(), WSLError> {
        self.inner.shutdown().await
    }

    async fn import(
        &self,
        name: &str,
        tar_path: &Path,
        install_path: &Path,
    ) -> Result<(), WSLError> {
        self.inner.import(name, tar_path, install_path).await
    }

    async fn export(&self, distro: &str, output: &Path) -> Result<(), WSLError> {
        self.inner.export(distro, output).await
    }

    async fn run_custom_action(
        &self,
        distro: &str,
        command: &str,
        output_tx: Sender<String>,
        input_rx: Receiver<String>,
    ) -> Result<(), WSLError> {
        self.inner
            .run_custom_action(distro, command, output_tx, input_rx)
            .await
    }
}
