use crossterm::style::style;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use crate::ui::screens::modals::centered_rect;

fn keybind(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {} ", key),
                     Style::default()
                         .bg(Color::Rgb(80, 80, 80))
                         .fg(Color::White)
                         .add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(desc.to_string(), Style::default().fg(Color::Gray)),
    ])
}
pub fn render_help(frame: &mut Frame<'_>) {
    let pop_up = centered_rect(60, 70, frame.area());

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("Keyboard Shortcuts", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        keybind("h", "Open help"),
        keybind("q", "Quit application"),
        keybind("r", "Run distro"),
        keybind("t", "Terminate Distro"),
        keybind("Enter", "Open shell"),
        keybind("d", "Set default distro"),
        keybind("e", "Export distro"),
        keybind("i", "Import distro"),
        Line::from(""),
        Line::from(Span::styled("ESC to close", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)))
    ];

    let help = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(
            Block::default()
        );
    frame.render_widget(Clear, pop_up);
    frame.render_widget(help, pop_up);
}