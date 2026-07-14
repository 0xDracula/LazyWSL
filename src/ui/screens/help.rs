use crate::app::keymaps::{self, action_bindings, display_keys};
use crate::config::KeymapConfig;
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

fn section_color(section: &str) -> Color {
    match section {
        "Navigation" => theme::ACCENT,
        "Distro" => theme::RUNNING,
        "Snapshots" => theme::ACCENT_ALT,
        "Danger Zone" => theme::ERROR,
        _ => theme::ACCENT,
    }
}

fn action_color(section: &str, label: &str) -> Color {
    match (section, label) {
        ("Danger Zone", _) => theme::ERROR,
        ("Snapshots", "Snapshot manager") => theme::INSTALLING,
        ("Snapshots", _) => theme::ACCENT_ALT,
        ("Distro", "Terminate distro") => theme::ERROR,
        ("Distro", "Pin distro") => theme::ACCENT_ALT,
        ("Distro", _) => theme::ACCENT,
        ("Navigation", "Open this help") => theme::STOPPED,
        ("Navigation", "Health check") => theme::ACCENT_ALT,
        ("Navgiation", _) => theme::ACCENT,
        _ => theme::ACCENT,
    }
}

pub fn render_help(frame: &mut Frame<'_>, keymaps: &KeymapConfig) {
    let pop_up = centered_rect(64, 88, frame.area());

    let mut lines = vec![Line::from("")];
    let mut current_section = "";

    for binding in action_bindings(keymaps) {
        if binding.section != current_section {
            if !current_section.is_empty() {
                lines.push(Line::from(""));
            }

            current_section = binding.section;
            lines.push(section(current_section, section_color(current_section)));
            lines.push(Line::from(""));
        }

        lines.push(keybind(
            &display_keys(binding.keys),
            binding.label,
            action_color(binding.section, binding.label),
        ));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Esc to close",
        Style::default()
            .fg(theme::TEXT_DIM)
            .add_modifier(Modifier::BOLD),
    )));

    let help = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(theme::modal_block("  Help").padding(Padding::horizontal(2)));
    frame.render_widget(Clear, pop_up);
    frame.render_widget(help, pop_up);
}
