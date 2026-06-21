use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};

use crate::{app::AppState, ui::theme};

pub fn render(frame: &mut Frame<'_>, state: &mut AppState, area: Rect) {
    let indices = state.filtered_indices();

    let max_size = state
        .distributions
        .iter()
        .filter_map(|d| d.size_bytes)
        .max()
        .unwrap_or(1)
        .max(1);

    let gauge_w = (area.width as usize).saturating_sub(20).clamp(6, 16);

    let mut items: Vec<ListItem> = Vec::new();

    if indices.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  no distro matched the search",
            theme::dim(),
        ))));
    } else {
        for &i in &indices {
            let d = &state.distributions[i];
            let is_pinned = state.pinned.contains(&d.name);
            let is_marked = state.selected_multi.contains(&d.name);

            let mut l1 = vec![
                Span::styled(
                    format!(" {} ", if is_marked { theme::MARKED } else { " " }),
                    Style::default().fg(theme::RUNNING),
                ),
                Span::styled(
                    theme::led(&d.state),
                    Style::default().fg(theme::state_color(&d.state)),
                ),
                Span::raw(" "),
                Span::styled(
                    theme::distro_icon(&d.name),
                    Style::default().fg(theme::ACCENT),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:<14}", d.name),
                    Style::default()
                        .fg(theme::TEXT)
                        .add_modifier(Modifier::BOLD),
                ),
            ];

            if d.is_default {
                l1.push(Span::styled(
                    format!(" {}", theme::DEFAULT_MARK),
                    Style::default().fg(theme::ACCENT),
                ));
            }

            if is_pinned {
                l1.push(Span::styled(
                    format!(" {}", theme::PIN),
                    Style::default().fg(theme::ACCENT_ALT),
                ));
            }

            let frac = d
                .size_bytes
                .map(|s| s as f64 / max_size as f64)
                .unwrap_or(0.0);
            let bar = theme::gauge_bar(frac, gauge_w);
            let size_txt = d
                .size_bytes
                .map(theme::format_size_short)
                .unwrap_or_else(|| "-".to_string());

            let l2 = vec![
                Span::styled(format!("     WSL{:<2} ", d.version), theme::dim()),
                Span::styled(bar, Style::default().fg(theme::state_color(&d.state))),
                Span::raw(" "),
                Span::styled(size_txt, theme::value()),
            ];

            items.push(ListItem::new(vec![Line::from(l1), Line::from(l2)]));
        }
    }

    let search_txt = if state.search_query.is_empty() {
        format!("{} /", theme::SEARCH)
    } else {
        format!("{} {}", theme::SEARCH, state.search_query)
    };

    let cursor = if state.search_active { "▍" } else { "" };

    let top_title = Line::from(vec![
        Span::styled(" DISTROS ", theme::label()),
        if state.busy {
            Span::styled("· busy", Style::default().fg(theme::INSTALLING))
        } else {
            Span::raw("")
        },
    ]);

    let bottom_title = Line::from(vec![
        Span::styled(format!(" {search_txt} "), theme::dim()),
        Span::styled(cursor, Style::default().fg(theme::ACCENT)),
        Span::raw(" "),
    ]);

    let block = Block::default()
        .title(top_title)
        .title_bottom(bottom_title)
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_active());

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(theme::SELECT_BG)
            .add_modifier(Modifier::BOLD),
    );

    let mut list_state = ListState::default();
    if !indices.is_empty() {
        list_state.select(Some(state.selected));
    }
    frame.render_stateful_widget(list, area, &mut list_state);

    let rows_visible = (area.height.saturating_sub(2) / 2) as usize;
    if indices.len() > rows_visible {
        let mut sb_state = ScrollbarState::new(indices.len()).position(state.selected);
        let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .style(theme::border());
        frame.render_stateful_widget(sb, area, &mut sb_state);
    }
}
