use ratatui::style::{Color, Modifier, Style};

use crate::wsl::{
    DistroState::{self, Stopped},
    WslVersion,
};

pub const BG: Color = Color::Rgb(13, 17, 23);
pub const SURFACE: Color = Color::Rgb(22, 27, 34);
pub const SELECT_BG: Color = Color::Rgb(31, 41, 55);

pub const BORDER: Color = Color::Rgb(48, 54, 61);
pub const BORDER_ACTIVE: Color = Color::Rgb(88, 166, 255);

pub const RUNNING: Color = Color::Rgb(63, 185, 80);
pub const STOPPED: Color = Color::Rgb(110, 118, 129);
pub const INSTALLING: Color = Color::Rgb(210, 153, 34);
pub const ERROR: Color = Color::Rgb(248, 81, 73);

pub const ACCENT: Color = Color::Rgb(88, 166, 255);
pub const ACCENT_ALT: Color = Color::Rgb(188, 140, 255);
pub const TEXT: Color = Color::Rgb(230, 237, 243);
pub const TEXT_DIM: Color = Color::Rgb(139, 148, 158);

pub const LED_ON: &str = "●";
pub const LED_OFF: &str = "○";
pub const PIN: &str = "*";
pub const DEFAULT_MARK: &str = "◆";
pub const MARKED: &str = "✔";
pub const SEARCH: &str = "\u{f002}";
pub const GAUGE_FULL: &str = "█";
pub const GAUGE_EMPTY: &str = "░";
pub const GAUGE_L: &str = "▕";
pub const GAUGE_R: &str = "▏";

pub fn distro_icon(name: &str) -> &'static str {
    let n = name.to_lowercase();
    if n.contains("ubuntu") {
        "\u{f31b}"
    } else if n.contains("debian") {
        "\u{f306}"
    } else if n.contains("kali") {
        "\u{f327}"
    } else if n.contains("alpine") {
        "\u{f300}"
    } else if n.contains("arch") {
        "\u{f303}"
    } else if n.contains("fedora") {
        "\u{f30a}"
    } else if n.contains("suse") {
        "\u{f314}"
    } else if n.contains("docker") {
        "\u{f308}"
    } else {
        "\u{f17c}"
    }
}

pub fn state_color(state: &DistroState) -> Color {
    match state {
        DistroState::Running => RUNNING,
        DistroState::Stopped => STOPPED,
        DistroState::Installing => INSTALLING,
        DistroState::Unknown(_) => STOPPED,
    }
}

pub fn led(state: &DistroState) -> &'static str {
    match state {
        DistroState::Running => LED_ON,
        _ => LED_OFF,
    }
}

pub fn dim() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn label() -> Style {
    Style::default().fg(TEXT_DIM).add_modifier(Modifier::BOLD)
}

pub fn value() -> Style {
    Style::default().fg(TEXT)
}

pub fn accent_bold() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn border() -> Style {
    Style::default().fg(BORDER)
}

pub fn border_active() -> Style {
    Style::default().fg(BORDER_ACTIVE)
}

pub fn chip(bg: Color) -> Style {
    Style::default().fg(BG).bg(bg).add_modifier(Modifier::BOLD)
}

pub fn gauge_bar(fraction: f64, width: usize) -> String {
    let inner = width.saturating_sub(2).max(1);
    let filled = ((fraction.clamp(0.0, 1.0)) * inner as f64).round() as usize;
    let filled = filled.min(inner);
    let empty = inner - filled;
    format!(
        "{}{}{}{}",
        GAUGE_L,
        GAUGE_FULL.repeat(filled),
        GAUGE_EMPTY.repeat(empty),
        GAUGE_R
    )
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

pub fn format_size_short(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0}K", bytes as f64 / KB as f64)
    } else {
        format!("{bytes}B")
    }
}

pub fn version_label(v: &WslVersion) -> String {
    format!("WSL {v}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauge_bar_widths() {
        assert_eq!(gauge_bar(0.0, 12).chars().filter(|c| *c == '█').count(), 0);
        assert_eq!(gauge_bar(1.0, 12).chars().filter(|c| *c == '█').count(), 10);
        assert_eq!(gauge_bar(0.5, 12).chars().filter(|c| *c == '█').count(), 5);
    }

    #[test]
    fn gauge_clamps_out_of_range() {
        assert_eq!(gauge_bar(2.0, 12).chars().filter(|c| *c == '█').count(), 10);
        assert_eq!(gauge_bar(-1.0, 12).chars().filter(|c| *c == '█').count(), 0);
    }

    #[test]
    fn short_size() {
        assert_eq!(format_size_short(8 * 1024 * 1024 * 1024), "8.0G");
        assert_eq!(format_size_short(512), "512B");
    }
}
