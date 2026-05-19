use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Span;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
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

        Modal::ActionOuptut { distro, action_name, output, finished, input, .. } => {
            let popup = centered_rect(85, 75, frame.area());
            let visible_lines = popup.height.saturating_sub(4) as usize;
            let output_parts: Vec<&str> = output.lines().collect();
            let start = output_parts.len().saturating_sub(visible_lines);
            let mut output_lines = vec![
                Line::from(vec![
                    Span::styled("Distro: ", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                    Span::styled(distro.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::styled("Action: ", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                    Span::styled(action_name.clone(), Style::default().fg(Color::White)),
                ]),
                Line::from(""),
            ];

            output_lines.extend(
                output_parts[start..].iter().map(|line| Line::from(Span::raw((*line).to_string()))),
            );

            if !output.ends_with('\n') && !output.is_empty() && output_parts.is_empty() {
                output_lines.push(Line::from(Span::raw(output.clone())));
            }

            output_lines.push(Line::from(""));

            if !*finished {
                output_lines.push(Line::from(vec![
                    Span::styled("Input: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("*".repeat(input.chars().count()), Style::default().fg(Color::Cyan))
                ]));
            }

            output_lines.push(Line::from(Span::styled(
                if *finished {
                    "Finished - Press ESC or q to close"
                } else {
                    "Running..."
                },
                Style::default().fg(if *finished {
                    Color::Green
                } else {
                    Color::Yellow
                })
            )));

            let para = Paragraph::new(output_lines).block(
                Block::default().borders(Borders::ALL).title(" Action Output ")
            );
            frame.render_widget(Clear, popup);
            frame.render_widget(para, popup);
        }

        Modal::CustomActionsMenu { distro, actions, selected } => {
            let popup = centered_rect(70, 55, frame.area());
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Distro: ", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                    Span::styled(distro.clone(), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(""),
            ];

            for (i, action) in actions.iter().enumerate() {
                let marker = if i == *selected { "> " } else { " " };
                let style = if i == *selected {
                    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                lines.push(Line::from(vec![
                    Span::styled(format!("{marker}"), style),
                    Span::styled(format!("{:<16}", action.name.clone()), style),
                    Span::styled(" ", style),
                    Span::styled(format!("{:<16}", action.command.clone()), Style::default().fg(Color::DarkGray)),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Enter to run - Esc to close",
                Style::default().fg(Color::DarkGray),
            )));

            let para = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title(" Custom Actions "));

            frame.render_widget(Clear, popup);
            frame.render_widget(para, popup);
        }
        
    }
}