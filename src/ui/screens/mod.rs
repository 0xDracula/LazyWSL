mod main;
mod help;
mod modals;

use ratatui::Frame;
use crate::app::AppState;

pub fn render(frame: &mut Frame<'_>, state: &mut AppState) {
    main::render_main(frame, state);
    modals::render_modals(frame, state);
}