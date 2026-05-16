use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState};
use crate::app::AppState;

pub fn render(frame: &mut Frame<'_>, state: &mut AppState, area: Rect) {
    let items: Vec<ListItem> = state
        .distributions
        .iter()
        .map(|d| {
            let def = if d.is_default { "●" } else { "○" };
            let line = Line::from(vec![
                Span::styled(format!("{def} "), Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:<16}", d.name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:<12} ", d.state),
                    Style::default().fg(
                        if d.state.to_string() == "Running" {
                            Color::Green
                        } else {
                            Color::DarkGray
                        }
                    ),
                ),
                Span::styled(format!("WSL {}", d.version), Style::default().fg(Color::Gray)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = if state.busy {
        "  LazyWSL - WSL distributions [busy...] "
    } else {
        "  LazyWSL - WSL distributions "
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(120, 180, 255))),

        )
        .highlight_style(Style::default().bg(Color::Rgb(40, 40, 50)).add_modifier(Modifier::BOLD))
        .highlight_symbol("▌ ");

    let mut list_state = ListState::default();
    if !state.distributions.is_empty() {
        list_state.select(Some(state.selected));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}