use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::Direction;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use crate::app::AppState;
use crate::core::DistroState;
use crate::ui::widgets::{distro_list, status};

pub fn render_main(frame: &mut Frame<'_>, state: &mut AppState) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(4),
            Constraint::Length(4),
            Constraint::Length(1),
        ])
        .split(area);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ]).split(chunks[0]);

    distro_list::render(frame, state, main_chunks[0]);

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

    let details = status::details_widget(state, color);
    frame.render_widget(details, main_chunks[1]);

    let status = status::status_widget(state);
    frame.render_widget(status, chunks[1]);

    let help = Paragraph::new(
        "h help | r run distro | Enter shell | t terminate | d default | u unregister | s shutdown | q/Esc quit"
    ).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[2]);
}