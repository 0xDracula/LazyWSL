use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear, FrameExt, Paragraph};
use crate::app::{AppState, Modal};
use crate::ui::screens::help::render_help;

pub fn centered_rect(x: u16, y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - y) / 2),
            Constraint::Percentage(y),
            Constraint::Percentage((100 - y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - x) / 2),
            Constraint::Percentage(x),
            Constraint::Percentage((100 - x) / 2),
        ])
        .split(vertical[1])[1]
}

pub fn render_modals(frame: &mut Frame<'_>, state: &mut AppState) {
    match &mut state.modal {
        Modal::None => {}
        Modal::Help => render_help(frame),

        Modal::ConfirmUnregister { name } => {
            let pop = centered_rect(60, 25, frame.area());
            let msg = format!("Unregister `{name}`?\n\nThis removes the distro and its files.");
            let para = Paragraph::new(msg).block(Block::default().borders(Borders::ALL).title(" Confirm "));
            frame.render_widget(Clear, pop);
            frame.render_widget(para, pop);
        }

        Modal::ConfirmShutdown => {
            let pop = centered_rect(60, 25, frame.area());
            let msg = "Shut down the entire WSL VMs?";
            let para = Paragraph::new(msg).block(Block::default().borders(Borders::ALL).title(" Confirm "));
            frame.render_widget(Clear, pop);
            frame.render_widget(para, pop);
        }

        Modal::ExportPicker { explorer, .. } => {
            let popup = centered_rect(80, 80, frame.area());
            frame.render_widget(Clear, popup);
            frame.render_widget_ref(explorer.widget(), popup);
        }

        Modal::ImportNameInput { input, .. } => {
            let pop_up = centered_rect(50, 20, frame.area());
            let paragraph = Paragraph::new(format!("Distro name: \n\n{}", input))
                .block(Block::default().borders(Borders::ALL).title(" Import "));
            frame.render_widget(Clear, pop_up);
            frame.render_widget(paragraph, pop_up);
        }

        Modal::ImportTarPicker { explorer }
        | Modal::ImportInstallPicker { explorer, .. } => {
            let popup = centered_rect(80, 80, frame.area());
            frame.render_widget(Clear, popup);
            frame.render_widget_ref(explorer.widget(), popup);
        }
    }
}