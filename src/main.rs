mod wsl;
mod app;
mod ui;

fn main() {
    for d in &wsl::get_distros().unwrap() {
        println!("{} - {} - {} - {}", d.name, d.is_default, d.version, d.state);
    }
}