use crate::app::diagnostics::DiagnosticLevel;
use crate::app::{AppState, Modal, snapshots};
use crate::ui::screens::help::render_help;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Span;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Clear, FrameExt, Padding, Paragraph};

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
        Modal::HealthCheck { report } => {
            let popup = centered_rect(78, 64, frame.area());
            frame.render_widget(Clear, popup);

            let block = theme::modal_block(" WSL Health Check (Esc/q to close) ")
                .padding(Padding::new(1, 1, 1, 1));

            let inner = block.inner(popup);
            frame.render_widget(block, popup);

            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Summary: ", theme::label()),
                    Span::styled(report.summary.clone(), theme::value()),
                ]),
                Line::from(""),
            ];

            for item in &report.items {
                let (icon, style) = match item.level {
                    DiagnosticLevel::Ok => ("✓", Style::default().fg(theme::RUNNING)),
                    DiagnosticLevel::Warning => ("⚠", Style::default().fg(Color::Yellow)),
                    DiagnosticLevel::Error => ("✗", Style::default().fg(theme::ERROR)),
                };

                lines.push(Line::from(vec![
                    Span::styled(format!("{icon}  "), style),
                    Span::styled(format!("{:<18}", item.label), theme::value()),
                    Span::styled(item.detail.clone(), theme::dim()),
                ]));
            }

            frame.render_widget(Paragraph::new(lines), inner);
        }
        Modal::ConfirmUnregister { names } => {
            let pop = centered_rect(60, 25, frame.area());
            let msg = if names.len() == 1 {
                format!(
                    "Unregister `{}`?\n\nThis removes the distro and its files",
                    names[0]
                )
            } else {
                format!(
                    "Unregister {} distros?\n\nThis removes the distros and their files\n\nPress y to confirm, n to cancel.",
                    names.len()
                )
            };
            let para = Paragraph::new(msg).block(theme::modal_block_warn("Confirm Unregister"));
            frame.render_widget(Clear, pop);
            frame.render_widget(para, pop);
        }

        Modal::ConfirmShutdown => {
            let pop = centered_rect(60, 25, frame.area());
            let msg = "Shut down the entire WSL VMs?\n\nPress y to confirm, n to cancel.";
            let para = Paragraph::new(msg).block(theme::modal_block_warn("Confirm Shutdown"));
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
                .block(theme::modal_block("Import"));
            frame.render_widget(Clear, pop_up);
            frame.render_widget(paragraph, pop_up);
        }

        Modal::ImportTarPicker { explorer } | Modal::ImportInstallPicker { explorer, .. } => {
            let popup = centered_rect(80, 80, frame.area());
            frame.render_widget(Clear, popup);
            frame.render_widget_ref(explorer.widget(), popup);
        }

        Modal::ActionOutput {
            distro,
            action_name,
            output,
            finished,
            input,
            ..
        } => {
            let popup = centered_rect(85, 75, frame.area());
            let visible_lines = popup.height.saturating_sub(4) as usize;
            let output_parts: Vec<&str> = output.lines().collect();
            let start = output_parts.len().saturating_sub(visible_lines);
            let mut output_lines = vec![
                Line::from(vec![
                    Span::styled(
                        "Distro: ",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(distro.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::styled(
                        "Action: ",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(action_name.clone(), Style::default().fg(Color::White)),
                ]),
                Line::from(""),
            ];

            output_lines.extend(
                output_parts[start..]
                    .iter()
                    .map(|line| Line::from(Span::raw((*line).to_string()))),
            );

            if !output.ends_with('\n') && !output.is_empty() && output_parts.is_empty() {
                output_lines.push(Line::from(Span::raw(output.clone())));
            }

            output_lines.push(Line::from(""));

            if !*finished {
                output_lines.push(Line::from(vec![
                    Span::styled("Input: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        "*".repeat(input.chars().count()),
                        Style::default().fg(Color::Cyan),
                    ),
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
                }),
            )));

            let para = Paragraph::new(output_lines).block(theme::modal_block("Action Output"));
            frame.render_widget(Clear, popup);
            frame.render_widget(para, popup);
        }

        Modal::CustomActionsMenu {
            distro,
            actions,
            selected,
        } => {
            let popup = centered_rect(70, 55, frame.area());
            let mut lines = vec![
                Line::from(vec![
                    Span::styled(
                        "Distro: ",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(distro.clone(), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(""),
            ];

            for (i, action) in actions.iter().enumerate() {
                let marker = if i == *selected { "> " } else { " " };
                let style = if i == *selected {
                    theme::modal_selected()
                } else {
                    theme::modal_row()
                };

                lines.push(Line::from(vec![
                    Span::styled(marker.to_string(), style),
                    Span::styled(format!("{:<16}", action.name.clone()), style),
                    Span::styled(" ", style),
                    Span::styled(
                        format!("{:<16}", action.command.clone()),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Enter to run - Esc to close",
                Style::default().fg(Color::DarkGray),
            )));

            let para = Paragraph::new(lines).block(theme::modal_block("Custom Actions"));

            frame.render_widget(Clear, popup);
            frame.render_widget(para, popup);
        }
        Modal::CloneDistro { distro, new_name } => {
            let pop_up = centered_rect(55, 25, frame.area());
            let lines = vec![
                Line::from(vec![
                    Span::styled(
                        "Clone: ",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(distro.clone(), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "New name: ",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(new_name.clone(), Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Enter to Clone, Esc to Cancel",
                    Style::default().fg(Color::DarkGray),
                )),
            ];

            let para = Paragraph::new(lines).block(theme::modal_block("Clone Distro"));

            frame.render_widget(Clear, pop_up);
            frame.render_widget(para, pop_up);
        }

        Modal::RollBackDistroPicker { distros, selected } => {
            let popup = centered_rect(70, 55, frame.area());
            let mut lines = vec![Line::from(Span::styled(
                "Select distro to restore",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ))];

            lines.push(Line::from(""));

            for (i, name) in distros.iter().enumerate() {
                let marker = if i == *selected { "> " } else { " " };
                let style = if i == *selected {
                    theme::modal_selected()
                } else {
                    theme::modal_row()
                };

                lines.push(Line::from(vec![
                    Span::styled(marker.to_string(), style),
                    Span::styled(name.clone(), style),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Enter to selected - Esc to clone",
                Style::default().fg(Color::DarkGray),
            )));

            let para = Paragraph::new(lines).block(theme::modal_block("Rollback"));

            frame.render_widget(Clear, popup);
            frame.render_widget(para, popup);
        }

        Modal::RollBackSnapShotPicker {
            distro,
            snapshots,
            selected,
        } => {
            let popup = centered_rect(80, 60, frame.area());
            let mut lines = vec![Line::from(vec![
                Span::styled(
                    "Distro: ",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(distro.clone(), Style::default().fg(Color::Cyan)),
            ])];
            lines.push(Line::from(""));

            for (i, p) in snapshots.iter().enumerate() {
                let name = p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("<snapsshot>")
                    .to_string();

                let marker = if i == *selected { "> " } else { " " };
                let style = if i == *selected {
                    theme::modal_selected()
                } else {
                    theme::modal_row()
                };

                lines.push(Line::from(vec![
                    Span::styled(marker.to_string(), style),
                    Span::styled(name, style),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Enter to confirm - Esc to cancel",
                Style::default().fg(Color::DarkGray),
            )));

            let para = Paragraph::new(lines).block(theme::modal_block("Choose snapshot"));

            frame.render_widget(Clear, popup);
            frame.render_widget(para, popup);
        }

        Modal::ConfirmRollBack {
            distro,
            snapshot,
            exists,
        } => {
            let pop = centered_rect(70, 25, frame.area());
            let snap_name = snapshot
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("<snapshot>");

            let msg = if *exists {
                format!(
                    "Rollback `{distro}` from `{snap_name}`?\n\nThis will DELETE the existing distro first.\n\n Press y to confirm, n to cancel."
                )
            } else {
                format!(
                    "Restore `{distro}` from `{snap_name}`?\n\nA new distro will be imported\n\nPress y to confirm, n to cancel."
                )
            };

            let para = Paragraph::new(msg).block(theme::modal_block_warn("Confirm Rollback"));

            frame.render_widget(Clear, pop);
            frame.render_widget(para, pop);
        }
        Modal::SnapshotManager {
            distros,
            distro_idx,
            snapshots,
            snap_idx,
            focus_right,
        } => {
            let popup = centered_rect(85, 70, frame.area());
            frame.render_widget(Clear, popup);

            let outer = theme::modal_block(
                " Snapshot Manager  (←/→ panes · x delete · p prune · Enter rollback · Esc) ",
            )
            .padding(Padding::new(1, 1, 1, 1));

            let inner = outer.inner(popup);
            frame.render_widget(outer, popup);

            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .spacing(1)
                .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
                .split(inner);

            let mut left_lines = vec![];
            for (i, d) in distros.iter().enumerate() {
                let total = snapshots::distro_snapshot_size(d);
                let selected = i == *distro_idx;
                let style = if selected && !*focus_right {
                    theme::modal_selected()
                } else if selected {
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD)
                } else {
                    theme::modal_row()
                };
                let marker = if selected { "> " } else { " " };
                left_lines.push(Line::from(vec![
                    Span::styled(marker.to_string(), style),
                    Span::styled(d.clone(), style),
                    Span::styled(
                        format!("  ({})", snapshots::format_size(total)),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }

            let left = Paragraph::new(left_lines).block(
                theme::modal_block("Distros")
                    .border_style(Style::default().fg(theme::BORDER))
                    .padding(Padding::new(1, 1, 1, 0)),
            );
            frame.render_widget(left, cols[0]);

            let mut right_lines = vec![];
            if snapshots.is_empty() {
                right_lines.push(Line::from(Span::styled(
                    "No snapshots for this distro.",
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                for (i, s) in snapshots.iter().enumerate() {
                    let selected = i == *snap_idx;
                    let style = if selected && *focus_right {
                        theme::modal_selected()
                    } else if selected {
                        Style::default().fg(theme::ACCENT)
                    } else {
                        theme::modal_row()
                    };

                    let marker = if selected { "> " } else { " " };

                    let age = s
                        .modified_secs
                        .map(format_age)
                        .unwrap_or_else(|| "?".to_string());

                    right_lines.push(Line::from(vec![
                        Span::styled(marker.to_string(), style),
                        Span::styled(s.file_name.clone(), style),
                        Span::styled(
                            format!("  {} {}", snapshots::format_size(s.size_bytes), age),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));
                }
            }
            let total = snapshots::total_snapshot_size();
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(Span::styled(
                format!(
                    "Total snapshot disk usage: {}",
                    snapshots::format_size(total)
                ),
                Style::default().fg(Color::Yellow),
            )));
            let right = Paragraph::new(right_lines).block(
                theme::modal_block("Snapshots")
                    .border_style(Style::default().fg(theme::BORDER))
                    .padding(Padding::new(1, 1, 1, 0)),
            );

            frame.render_widget(right, cols[1]);
        }
        Modal::ConfirmDeleteSnapshot { snapshot, .. } => {
            let pop = centered_rect(70, 25, frame.area());
            let name = snapshot
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("<snapshot>");

            let msg =
                format!("Delete snapshot: `{name}`?\n\nThis permanently removes the snapshot");
            let para = Paragraph::new(msg).block(theme::modal_block_warn("Confirm Delete"));
            frame.render_widget(Clear, pop);
            frame.render_widget(para, pop);
        }

        Modal::ConfirmPruneSnapshots { distro, keep, .. } => {
            let pop = centered_rect(70, 25, frame.area());
            let msg = format!(
                "Prune snapshots for `{distro}`?\n\nKeeps the newest {keep}, deletes the rest!\n\nPress y to confirm, n to cancel"
            );
            let para = Paragraph::new(msg).block(theme::modal_block_warn("Confirm Prune"));
            frame.render_widget(Clear, pop);
            frame.render_widget(para, pop);
        }

        Modal::CatalogLoading => {
            let pop = centered_rect(50, 20, frame.area());
            frame.render_widget(Clear, pop);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "  Fetching catalog...",
                    theme::dim(),
                )))
                .block(theme::modal_block("Online catalog")),
                pop,
            );
        }

        Modal::CatalogPicker {
            entries,
            filtered,
            selected,
            query,
        } => {
            let popup = centered_rect(80, 70, frame.area());
            frame.render_widget(Clear, popup);
            let block = theme::modal_block("Install a distro").padding(Padding::new(1, 1, 1, 1));
            let inner = block.inner(popup);
            frame.render_widget(block, popup);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(4)])
                .split(inner);

            let q = if query.is_empty() {
                "type to search".to_string()
            } else {
                query.clone()
            };
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(
                        format!("{}  ", theme::SEARCH),
                        Style::default().fg(theme::ACCENT),
                    ),
                    Span::styled(q, theme::dim()),
                ])),
                rows[0],
            );

            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .spacing(1)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(rows[1]);

            let mut left = vec![];
            for (row, &idx) in filtered.iter().enumerate() {
                let e = &entries[idx];
                let sel = row == *selected;
                let style = if sel {
                    theme::modal_selected()
                } else {
                    theme::modal_row()
                };
                left.push(Line::from(vec![
                    Span::styled(if sel { " > " } else { "  " }.to_string(), style),
                    Span::styled(
                        theme::distro_icon(&e.name),
                        Style::default().fg(theme::ACCENT),
                    ),
                    Span::raw(" "),
                    Span::styled(e.friendly.clone(), style),
                ]));
            }

            if filtered.is_empty() {
                left.push(Line::from(Span::styled("  no match", theme::dim())));
            }

            frame.render_widget(
                Paragraph::new(left).block(
                    theme::modal_block("Catalog").border_style(Style::default().fg(theme::BORDER)),
                ),
                cols[0],
            );

            let mut right = vec![];
            if let Some(&idx) = filtered.get(*selected) {
                let e = &entries[idx];
                right.push(Line::from(vec![
                    Span::styled(
                        theme::distro_icon(&e.name),
                        Style::default().fg(theme::ACCENT),
                    ),
                    Span::raw(" "),
                    Span::styled(e.friendly.clone(), theme::value()),
                ]));
                right.push(Line::from(""));
                right.push(Line::from(vec![
                    Span::styled("id   ", theme::label()),
                    Span::styled(e.name.clone(), theme::value()),
                ]));
                right.push(Line::from(vec![
                    Span::styled("wsl   ", theme::label()),
                    Span::styled("2", Style::default().fg(theme::RUNNING)),
                ]));
                right.push(Line::from(""));
                right.push(Line::from(Span::styled(
                    "installs without launching",
                    theme::dim(),
                )));
                right.push(Line::from(Span::styled(
                    "run it once to finish setup",
                    theme::dim(),
                )));
            }

            frame.render_widget(
                Paragraph::new(right).block(
                    theme::modal_block("Details").border_style(Style::default().fg(theme::BORDER)),
                ),
                cols[1],
            );
        }

        Modal::ConfirmInstall { entry } => {
            let pop = centered_rect(70, 30, frame.area());
            let lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Install ", theme::value()),
                    Span::styled(entry.friendly.clone(), Style::default().fg(theme::ACCENT)),
                    Span::styled(" ?", theme::value()),
                ]),
                Line::from(Span::styled(format!("id: {}", entry.name), theme::dim())),
                Line::from(""),
                Line::from(Span::styled(
                    "Downlods several hundred MB. Installs",
                    theme::dim(),
                )),
                Line::from(Span::styled(
                    "without launching, run it once to set up",
                    theme::dim(),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" y ", theme::chip(theme::RUNNING)),
                    Span::styled(" confirm ", theme::dim()),
                    Span::styled(" n ", theme::chip(theme::ERROR)),
                    Span::styled(" cancel ", theme::dim()),
                ]),
            ];

            frame.render_widget(Clear, pop);
            frame.render_widget(
                Paragraph::new(lines).block(theme::modal_block_warn("Confirm Install")),
                pop,
            );
        }
    }
}

fn format_age(modified_secs: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(modified_secs);

    let delta = now.saturating_sub(modified_secs);
    match delta {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{}m ago", delta / 60),
        3600..=86399 => format!("{}h ago", delta / 3600),
        _ => format!("{}d ago", delta / 86400),
    }
}
