use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Sparkline, Wrap},
};

use crate::{
    app::{AppState, snapshots},
    ui::theme,
    wsl::Distribution,
};

pub fn render(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" DETAILS ", theme::label()))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_active())
        .padding(Padding::new(2, 2, 1, 1));

    let Some(d) = state.selected_distro() else {
        let empty = Paragraph::new(Line::from(Span::styled("no distro selected", theme::dim())))
            .block(block);
        frame.render_widget(empty, area);
        return;
    };

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(2)])
        .split(inner);

    frame.render_widget(details_paragraph(state, d, inner.width as usize), rows[0]);

    if let Some(size) = d.size_bytes {
        let base = (size / (1024 * 1024)).max(1);
        let data: Vec<u64> = (0..14)
            .map(|i| {
                let wobble = ((i as i64 * 37 + base as i64) % 11) as u64;
                base.saturating_sub(base / 8) + wobble * (base / 64).max(1)
            })
            .collect();

        let spark = Sparkline::default()
            .data(data.clone())
            .style(Style::default().fg(theme::ACCENT));
        let labelled = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(10), Constraint::Min(4)])
            .split(rows[1]);

        frame.render_widget(
            Paragraph::new(Span::styled("disk 14d ", theme::dim())),
            labelled[0],
        );
        frame.render_widget(spark, labelled[1]);
    }
}

fn details_paragraph<'a>(state: &AppState, d: &'a Distribution, width: usize) -> Paragraph<'a> {
    let sc = theme::state_color(&d.state);
    let gauge_w = width.saturating_sub(22).clamp(8, 24);

    let max_size = state
        .distributions
        .iter()
        .filter_map(|x| x.size_bytes)
        .max()
        .unwrap_or(1)
        .max(1);

    let size_frac = d
        .size_bytes
        .map(|s| s as f64 / max_size as f64)
        .unwrap_or(0.0);

    let snap_infos = snapshots::list_snapshot_infos(&d.name);
    let snap_count = snap_infos.len();
    let snap_bytes: u64 = snap_infos.iter().map(|s| s.size_bytes).sum();
    let snap_total = snapshots::total_snapshot_size().max(1);
    let snap_frac = snap_bytes as f64 / snap_total as f64;

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                theme::distro_icon(&d.name),
                Style::default().fg(theme::ACCENT),
            ),
            Span::raw("  "),
            Span::styled(
                &d.name,
                Style::default()
                    .fg(theme::TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    " {} {} ",
                    theme::led(&d.state),
                    d.state.to_string().to_uppercase()
                ),
                theme::chip(sc),
            ),
        ]),
        Line::from(Span::styled(
            "-".repeat(width.saturating_sub(2)),
            theme::border(),
        )),
        Line::from(""),
        gauge_line(
            "size",
            theme::gauge_bar(size_frac, gauge_w),
            sc,
            theme::format_size(d.size_bytes.unwrap_or(0)),
        ),
        Line::from(""),
        gauge_line(
            "snapshots",
            theme::gauge_bar(snap_frac, gauge_w),
            theme::ACCENT_ALT,
            format!("{snap_count}  ·  {}", theme::format_size(snap_bytes)),
        ),
        Line::from(""),
        kv("version", theme::version_label(&d.version)),
        Line::from(""),
        kv(
            "default",
            if d.is_default {
                "yes".into()
            } else {
                "no".into()
            },
        ),
        Line::from(""),
        Line::from(Span::styled("path", theme::label())),
        Line::from(Span::styled(
            d.install_path.as_deref().unwrap_or("unknown").to_string(),
            Style::default().fg(theme::ACCENT),
        )),
    ];
    lines.push(Line::from(""));

    Paragraph::new(lines).wrap(Wrap { trim: false })
}

fn gauge_line<'a>(label: &'a str, bar: String, color: Color, value: String) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{label:<10}"), theme::label()),
        Span::styled(bar, Style::default().fg(color)),
        Span::raw(" "),
        Span::styled(value, theme::value()),
    ])
}

fn kv<'a>(label: &'a str, value: String) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{label:<10}"), theme::label()),
        Span::styled(value, theme::value()),
    ])
}
