mod wsl;
mod app;
mod ui;

fn main() {

    // FOR TESTING PURPOSES

    // for d in &wsl::get_distros().unwrap() {
    //     println!("{} - {} - {} - {}", d.name, d.is_default, d.version, d.state);
    // }

    // wsl::shutdown("Ubuntu").unwrap();
    //
    // for d in &wsl::get_distros().unwrap() {
    //     println!("{} - {} - {} - {}", d.name, d.is_default, d.version, d.state);
    // }

    // wsl::set_default("Ubuntu").unwrap();
    // for d in &wsl::get_distros().unwrap() {
    //     println!("{} - {} - {} - {}", d.name, d.is_default, d.version, d.state);
    // }
}