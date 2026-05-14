use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use crate::app::AppState;

fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * KB;
    const GB: f64 = 1024.0 * MB;

    let b = bytes as f64;

    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.2} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

pub fn details_widget(state: &AppState, color: Color) -> Paragraph<'_> {
    let details_lines = if let Some(d) = state.selected_distro() {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Name: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(&d.name),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status: ", Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(d.state.to_string()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Version: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(format!("V{}", d.version)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Size: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(d.size_bytes.map(format_size).unwrap_or_else(|| "Unknown".to_string())),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Install Path: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    d.install_path.as_deref().unwrap_or("Unknown"),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ]
    } else {
        vec![Line::from("No distro selected!")]
    };

    Paragraph::new(details_lines)
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title(" Details "))
}

pub fn status_widget(state: &AppState) -> Paragraph<'static> {
    Paragraph::new(state.status_line.clone()).block(
        Block::default()
            .borders(Borders::ALL)
            .title("  Status  "),
    )
}