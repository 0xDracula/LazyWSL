use ratatree::FilePicker;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use crate::app::{AppState, Pending };
use wsl::DistroState;
use crate::wsl;

fn centered_rect(x: u16, y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage((100 - y) / 2),
            Constraint::Percentage(y), Constraint::Percentage((100 - y) / 2)]).split(area);

    Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage((100 - x) / 2),
        Constraint::Percentage(x), Constraint::Percentage((100 - x) / 2)]).split(vertical[1])[1]
}

pub fn format_size(bytes: u64) -> String {
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

fn render_help(frame: &mut Frame<'_>) {
    let pop_up = centered_rect(60, 70, frame.area());
    let lines = vec![
        Line::from(""),

        Line::from(vec![
            Span::styled(" h ", Style::default().fg(Color::Black).bg(Color::White)),

            Span::raw(" Help "),
        ]),

        Line::from(""),

        Line::from(vec![
            Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::White)),

            Span::raw(" Quit "),
        ]),

        Line::from(""),

        Line::from(vec![
            Span::styled(" r ", Style::default().fg(Color::Black).bg(Color::White)),

            Span::raw(" Run Distro "),
        ]),

        Line::from(""),

        Line::from(vec![
            Span::styled(" t ", Style::default().fg(Color::Black).bg(Color::White)),

            Span::raw(" Terminate Distro ")
        ]),

        Line::from(""),

        Line::from(vec![
            Span::styled(" Enter ", Style::default().fg(Color::Black).bg(Color::White)),

            Span::raw(" Enter Shell ")
        ]),

        Line::from(""),

        Line::from(vec![
            Span::styled(" d ", Style::default().fg(Color::Black).bg(Color::White)),

            Span::raw(" Set Default Distro "),
        ]),

        Line::from(""),

        Line::from(vec![
            Span::styled(" u ", Style::default().fg(Color::Black).bg(Color::White)),

            Span::raw(" Unregister a distro - Destructive Action "),
        ]),

        Line::from(""),

        Line::from(vec![
            Span::styled(" s ", Style::default().fg(Color::Black).bg(Color::White)),

            Span::raw(" Shutdown all distros "),
        ]),
    ];

    let help = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(Clear, pop_up);
    frame.render_widget(help, pop_up);
}

pub fn render(frame: &mut Frame<'_>, state: &mut AppState) {
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
            Constraint::Percentage(30),
            Constraint::Percentage(70),
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
                Span::styled("  Size: ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)), Span::raw(
                        d.size_byes.map(format_size).unwrap_or_else(|| { "Unknown".to_string() })
                    )
            ]),

            Line::from(""),

            Line::from(vec![
                Span::styled("  Install Path: ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )
            ]),

            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    d.install_path.as_deref().unwrap_or("Unknown"),
                    Style::default().fg(Color::Cyan),
                )
            ])
        ]
    } else {
         vec![Line::from("No distro selected!")]
    };

    let details = Paragraph::new(details_lines).wrap( Wrap { trim: false } ).block(Block::default().borders(Borders::ALL).title(" Details "));
    frame.render_widget(details, main_chunks[1]);
    let mut status_txt = state.status_line.clone();
    match &mut state.pending {
        Pending::None => {}
        Pending::Help => {}
        Pending::ConfirmUnregister { name } => {
            status_txt.push_str(&format!(
                "\n[y/n] Unregister `{name}`? This removes the distro and its files "
            ));
        }
        Pending::ConfirmShutdown => {
            status_txt.push_str("\n[y/n] Shut down the entire WSL VMs?");
        }
        Pending::ExportPicker { distro, picker } => {
            let popup = centered_rect(
                80,
                80,
                frame.area(),
            );

            frame.render_widget(
                Clear,
                popup,
            );

            frame.render_stateful_widget(
                FilePicker::default(),
                popup,
                picker,
            );
        }
    }

    let status = Paragraph::new(status_txt).block(
        Block::default()
            .borders(Borders::ALL)
            .title("  Status  "),
    );

    frame.render_widget(status, chunks[1]);

    let help = Paragraph::new(
        "h help | r run distro | Enter shell | t terminate | d default | u unregister | s shutdown | q/Esc quit"
    ).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[2]);

    if matches!(state.pending, Pending::Help) {
        render_help(frame);
    }
}