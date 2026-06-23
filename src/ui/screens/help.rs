use crate::ui::screens::modals::centered_rect;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Padding, Paragraph};

fn keybind(key: &str, desc: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {key} "), theme::chip(color)),
        Span::raw(" "),
        Span::styled(desc.to_string(), theme::value()),
    ])
}

fn section(title: &str, color: Color) -> Line<'static> {
    Line::from(Span::styled(
        title.to_string(),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
}

pub fn render_help(frame: &mut Frame<'_>) {
    let pop_up = centered_rect(64, 88, frame.area());

    let lines = vec![
        Line::from(""),
        section("Navigation", theme::ACCENT),
        Line::from(""),
        keybind("↑↓", "Move selection", theme::ACCENT),
        keybind("Space", "Multi-select", theme::ACCENT),
        keybind("/", "Search", theme::ACCENT),
        keybind("?", "Open this help", theme::STOPPED),
        Line::from(""),
        section("Distro", theme::RUNNING),
        Line::from(""),
        keybind("⏎", "Open shell", theme::RUNNING),
        keybind("r", "Run distro", theme::RUNNING),
        keybind("t", "Terminate distro", theme::ERROR),
        keybind("d", "Set default", theme::ACCENT),
        keybind("p", "Pin distro", theme::ACCENT_ALT),
        keybind("e", "Export distro", theme::ACCENT),
        keybind("i", "Import distro", theme::ACCENT),
        keybind("a", "Custom actions", theme::ACCENT),
        keybind("n", "Clone distro", theme::ACCENT),
        Line::from(""),
        section("Snapshots", theme::ACCENT_ALT),
        Line::from(""),
        keybind("z", "Snapshot distro", theme::ACCENT_ALT),
        keybind("b", "Rollback distro", theme::ACCENT_ALT),
        keybind("S", "Snapshot manager", theme::INSTALLING),
        Line::from(""),
        section("Danger Zone", theme::ERROR),
        Line::from(""),
        keybind("u", "Unregister distro", theme::ERROR),
        keybind("s", "Shutdown all distros", theme::ERROR),
        keybind("q", "Quit", theme::STOPPED),
        Line::from(""),
        Line::from(Span::styled(
            "Esc to close",
            Style::default()
                .fg(theme::TEXT_DIM)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    let help = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(theme::modal_block("  Help").padding(Padding::horizontal(2)));
    frame.render_widget(Clear, pop_up);
    frame.render_widget(help, pop_up);
}
