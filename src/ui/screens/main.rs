use crate::app::AppState;
use crate::ui::theme;
use crate::ui::widgets::{distro_list, footer, header, status};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::Direction;

pub fn render_main(frame: &mut Frame<'_>, state: &mut AppState) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    header::render(frame, state, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    distro_list::render(frame, state, body[0]);
    status::render(frame, state, body[1]);

    footer::render(frame, state, chunks[3]);

    let _ = theme::BG;
}
