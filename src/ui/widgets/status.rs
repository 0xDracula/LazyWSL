use ratatui::layout::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap};
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
            Line::from(
                Span::styled("System Information", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{:<12}", "Name:"), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                Span::raw(&d.name),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{:<12}", "Status:"), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(d.state.to_string()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{:<12}", "Version:"), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                Span::raw(format!("V{}", d.version)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{:<12}", "Size:"), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                Span::raw(d.size_bytes.map(format_size).unwrap_or_else(|| "Unknown".to_string())),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("{:<12}", "Install Path:"), Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
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
        .block(Block::default()
            .title(" Details ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(120, 180, 255)))
            .padding(Padding::horizontal(1))
        )
}

pub fn status_widget(state: &AppState) -> Paragraph<'static> {
    Paragraph::new(state.status_line.clone()).block(
        Block::default()
            .title(" Status ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(120, 180, 255)))
    )
}