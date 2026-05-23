use crate::app::AppState;
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState};

pub fn render(frame: &mut Frame<'_>, state: &mut AppState, area: Rect) {
    let indices = state.filtered_indices();
    let mut items: Vec<ListItem> = if indices.is_empty() {
        vec![ListItem::new(Line::from("No distro matched the search"))]
    } else {
        indices
            .iter()
            .map(|&i| {
                let d = &state.distributions[i];
                let is_pinned = state.pinned.contains(&d.name);
                let pin = if is_pinned { "★" } else { "" };
                let def = if d.is_default { "●" } else { "○" };
                let is_marked = state.selected_multi.contains(&d.name);
                let mark = if is_marked { "✔" } else { " " };
                let line = Line::from(vec![
                    Span::styled(format!("{mark} "), Style::default().fg(Color::Green)),
                    Span::styled(format!("{pin} "), Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{def} "), Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{:<16}", d.name),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:<12} ", d.state),
                        Style::default().fg(if d.state.to_string() == "Running" {
                            Color::Green
                        } else {
                            Color::DarkGray
                        }),
                    ),
                    Span::styled(
                        format!("WSL {}", d.version),
                        Style::default().fg(Color::Gray),
                    ),
                ]);
                ListItem::new(line)
            })
            .collect()
    };

    let search_label = if state.search_query.is_empty() {
        "Search: / to start".to_string()
    } else {
        format!("Search: {}", state.search_query)
    };

    let cursor = if state.search_active { "▍" } else { "" };
    let search_line = Line::from(vec![
        Span::styled(search_label, Style::default().fg(Color::DarkGray)),
        Span::styled(cursor, Style::default().fg(Color::Cyan)),
    ]);

    items.push(ListItem::new(search_line));

    let title = if state.busy {
        format!("  LazyWSL - [search: {}] [busy...]  ", state.search_query)
    } else {
        format!("  LazyWSL - [search: {}]  ", state.search_query)
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
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 40, 50))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▌ ");

    let mut list_state = ListState::default();
    if !indices.is_empty() {
        list_state.select(Some(state.selected));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}
