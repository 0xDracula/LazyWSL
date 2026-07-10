use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{app::AppState, ui::theme};

fn chip<'a>(key: &'a str, label: &'a str, color: Color) -> Vec<Span<'a>> {
    vec![
        Span::styled(format!(" {key} "), theme::chip(color)),
        Span::styled(format!(" {label} "), theme::dim()),
    ]
}

pub fn render(frame: &mut Frame<'_>, _state: &AppState, area: Rect) {
    let mut spans = Vec::new();
    spans.extend(chip("↑↓", "move", theme::ACCENT));
    spans.extend(chip("⏎", "shell", theme::RUNNING));
    spans.extend(chip("r", "run", theme::RUNNING));
    spans.extend(chip("t", "stop", theme::ERROR));
    spans.extend(chip("z", "snap", theme::ACCENT_ALT));
    spans.extend(chip("S", "mgr", theme::INSTALLING));
    spans.extend(chip("H", "health", theme::ACCENT_ALT));
    spans.extend(chip("?", "help", theme::STOPPED));

    let para = Paragraph::new(Line::from(spans));
    frame.render_widget(para, area);
}
