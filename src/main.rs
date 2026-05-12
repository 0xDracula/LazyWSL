use crate::wsl::WslProcess;

mod wsl;
mod app;
mod ui;
mod errors;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    app::run_tui().await
}