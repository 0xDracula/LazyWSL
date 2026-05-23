mod app;
mod config;
mod core;
mod ui;
mod wsl;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    let _cfg = config::load_or_create();
    app::run_tui().await
}
