mod help;
mod main;
mod modals;

use crate::app::AppState;
use ratatui::Frame;

pub fn render(frame: &mut Frame<'_>, state: &mut AppState) {
    main::render_main(frame, state);
    modals::render_modals(frame, state);
}
