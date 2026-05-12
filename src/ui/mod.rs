use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use crate::app::{AppState, Pending };

pub fn render(frame: &mut Frame<'_>, state: &AppState) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(4),
            Constraint::Length(4),
            Constraint::Length(1),
        ]).split(area);

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
        }).collect();

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

    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    let mut status_txt = state.status_line.clone();
    match &state.pending {
        Pending::None => {}
        Pending::ConfirmUnregister { name } => {
            status_txt.push_str(&format!(
                "\n[y/n] Unregister `{name}`? This removes the distro and its files "
            ));
        }
        Pending::ConfirmShutdown => {
            status_txt.push_str("\n[y/n] Shut down the entire WSL VMs?");
        }
    }

    let status = Paragraph::new(status_txt).block(
        Block::default()
            .borders(Borders::ALL)
            .title("  Status  "),
    );

    frame.render_widget(status, chunks[1]);

    let help = Paragraph::new(
        "r run distro | Enter shell | t terminate | d default | u unregister | s shutdown | q/Esc quit"
    ).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[2]);
}