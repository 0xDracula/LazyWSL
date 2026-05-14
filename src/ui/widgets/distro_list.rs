use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use crate::app::AppState;

pub fn render(frame: &mut Frame<'_>, state: &mut AppState, area: Rect) {
    let items: Vec<ListItem> = state
        .distributions
        .iter()
        .map(|d| {
            let def = if d.is_default { "*" } else { " " };
            let line = Line::from(vec![
                Span::raw(format!("{def} ")),
                Span::styled(&d.name, Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled(
                    d.state.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  WSL "),
                Span::styled(d.version.to_string(), Style::default().fg(Color::Green)),
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
                .borders(Borders::ALL)
                .title(title),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    if !state.distributions.is_empty() {
        list_state.select(Some(state.selected));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}