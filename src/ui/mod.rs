use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use thiserror::__private18::AsDisplay;
use crate::app::{AppState, Pending };
use wsl::DistroState;
use crate::wsl;

pub fn render(frame: &mut Frame<'_>, state: &AppState) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(4),
            Constraint::Length(4),
            Constraint::Length(1),
        ]).split(area);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(chunks[0]);

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

    frame.render_stateful_widget(list, main_chunks[0], &mut list_state);

    let color = if let Some(d) = state.selected_distro() {
        match &d.state {
            DistroState::Running => Color::Green,
            DistroState::Stopped => Color::Red,
            DistroState::Installing => Color::Yellow,
            DistroState::Unknown(_) => Color::Green,
        }
    } else {
        Color::White
    };

    let details_lines = if let Some(d) = state.selected_distro() {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Name: ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)), Span::raw(&d.name)
            ]),

            Line::from(""),

            Line::from(vec![
                Span::styled("  Status: ",
                Style::default().fg(color).add_modifier(Modifier::BOLD)), Span::raw(d.state.to_string()),
            ]),

            Line::from(""),

            Line::from(vec![
                Span::styled("  Version: ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)), Span::raw(
                    format!("V{}", d.version.clone().to_string())
                )
            ]),

            Line::from(""),

            Line::from(vec![
                Span::styled("  Install Path: ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)), Span::raw(
                    d.install_path.clone().unwrap_or_else(|| { "Unknown".to_string() })
                )
            ])
        ]
    } else {
         vec![Line::from("No distro selected!")]
    };

    let details = Paragraph::new(details_lines).wrap( Wrap { trim: false } ).block(Block::default().borders(Borders::ALL).title(" Details "));
    frame.render_widget(details, main_chunks[1]);
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