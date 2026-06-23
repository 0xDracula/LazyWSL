use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{
    app::{AppState, snapshots},
    ui::theme,
    wsl::DistroState,
};

pub fn render(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let total = state.distributions.len();
    let running = state
        .distributions
        .iter()
        .filter(|d| matches!(d.state, DistroState::Running))
        .count();

    let disk: u64 = state
        .distributions
        .iter()
        .filter_map(|d| d.size_bytes)
        .sum::<u64>()
        + snapshots::total_snapshot_size();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_active());

    let innner = block.inner(area);
    frame.render_widget(block, area);

    let line = Line::from(vec![
        Span::styled(" ▰ ▰ ▰  ", Style::default().fg(theme::ACCENT_ALT)),
        Span::styled(
            "LAZYWSL",
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   control panel", theme::dim()),
        Span::raw("        "),
        Span::styled("distros ", theme::dim()),
        Span::styled(total.to_string(), theme::value()),
        Span::raw("   "),
        Span::styled(theme::LED_ON, Style::default().fg(theme::RUNNING)),
        Span::styled(" running ", theme::dim()),
        Span::styled(running.to_string(), theme::value()),
        Span::raw("   "),
        Span::styled("disk  ", theme::dim()),
        Span::styled(
            theme::format_size(disk),
            Style::default().fg(theme::INSTALLING),
        ),
    ]);

    frame.render_widget(Paragraph::new(line).alignment(Alignment::Left), innner);

    if let Some(toast) = state.notifications.latest() {
        let color = toast.level.color();
        let right = Line::from(vec![
            Span::styled(
                format!("{} ", toast.level.icon()),
                Style::default().fg(color),
            ),
            Span::styled(
                toast.msg.clone(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ]);

        frame.render_widget(Paragraph::new(right).alignment(Alignment::Right), innner);
    }
}
