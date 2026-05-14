use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

fn centered_rect(x: u16, y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - y) / 2),
            Constraint::Percentage(y),
            Constraint::Percentage((100 - y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - x) / 2),
            Constraint::Percentage(x),
            Constraint::Percentage((100 - x) / 2),
        ])
        .split(vertical[1])[1]
}

pub fn render_help(frame: &mut Frame<'_>) {
    let pop_up = centered_rect(60, 70, frame.area());

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(" h ", Style::default().fg(Color::Black).bg(Color::White)), Span::raw(" Help ")]),
        Line::from(""),
        Line::from(vec![Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::White)), Span::raw(" Quit ")]),
        Line::from(""),
        Line::from(vec![Span::styled(" r ", Style::default().fg(Color::Black).bg(Color::White)), Span::raw(" Run Distro ")]),
        Line::from(""),
        Line::from(vec![Span::styled(" t ", Style::default().fg(Color::Black).bg(Color::White)), Span::raw(" Terminate Distro ")]),
        Line::from(""),
        Line::from(vec![Span::styled(" Enter ", Style::default().fg(Color::Black).bg(Color::White)), Span::raw(" Enter Shell ")]),
        Line::from(""),
        Line::from(vec![Span::styled(" d ", Style::default().fg(Color::Black).bg(Color::White)), Span::raw(" Set Default Distro ")]),
        Line::from(""),
        Line::from(vec![Span::styled(" u ", Style::default().fg(Color::Black).bg(Color::White)), Span::raw(" Unregister a distro - Destructive Action ")]),
        Line::from(""),
        Line::from(vec![Span::styled(" s ", Style::default().fg(Color::Black).bg(Color::White)), Span::raw(" Shutdown all distros ")]),
    ];

    let help = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(Clear, pop_up);
    frame.render_widget(help, pop_up);
}