use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::theme;
use super::super::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &mut App) {
    // Split vertically: table (top) + deliverable detail (bottom)
    let sel = app.milestone_table_state.selected().unwrap_or(0);
    let has_detail = sel < app.data.milestones.len()
        && app.data.milestones[sel].total_deliverables > 0;

    let chunks = if has_detail {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(0)])
            .split(area)
    };

    render_milestone_table(f, chunks[0], app);

    if has_detail && chunks[1].height > 0 {
        render_detail(f, chunks[1], app, sel);
    }
}

fn render_milestone_table(f: &mut Frame, area: Rect, app: &mut App) {
    let milestones = &app.data.milestones;

    let header = Row::new(vec![
        Cell::from("ID").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Deliverables").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Review").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().fg(Color::White).bg(Color::DarkGray));

    let rows: Vec<Row> = milestones
        .iter()
        .map(|m| {
            let pct = m.completion_pct();
            let deliverables_str = format!(
                "{}/{} ({:.0}%)",
                m.completed_deliverables,
                m.total_deliverables,
                pct * 100.0
            );

            let status_cell = if m.is_active {
                Cell::from(Span::styled("ACTIVE", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            } else if pct >= 1.0 {
                Cell::from(Span::styled("✓ DONE", Style::default().fg(theme::GREEN)))
            } else if pct > 0.0 {
                Cell::from(Span::styled("partial", Style::default().fg(theme::GOLD)))
            } else {
                Cell::from(Span::styled("—", Style::default().fg(Color::DarkGray)))
            };

            let id_style = if m.is_active {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if pct >= 1.0 {
                Style::default().fg(theme::GREEN)
            } else {
                Style::default().fg(Color::White)
            };

            let review_style = match m.review_status.as_str() {
                "RE-REVIEWED" => Style::default().fg(theme::BLUE),
                "REVIEWED" => Style::default().fg(theme::GREEN),
                _ => Style::default().fg(Color::DarkGray),
            };
            let review_text = if m.review_status.is_empty() { "—" } else { &m.review_status };

            Row::new(vec![
                Cell::from(Span::styled(m.id.clone(), id_style)),
                Cell::from(m.name.clone()),
                Cell::from(Span::styled(deliverables_str, Style::default().fg(Color::Gray))),
                Cell::from(Span::styled(review_text.to_owned(), review_style)),
                status_cell,
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Min(30),
        Constraint::Length(16),
        Constraint::Length(12),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Milestones "))
        .row_highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("→ ");

    f.render_stateful_widget(table, area, &mut app.milestone_table_state);
}

fn render_detail(f: &mut Frame, area: Rect, app: &App, sel: usize) {
    let m = &app.data.milestones[sel];

    // We can't easily get deliverable text from the parsed data (we only stored counts).
    // Show a summary instead.
    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(format!("{}: ", m.id), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&m.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  {}/{} deliverables complete  ({:.0}%)",
                    m.completed_deliverables, m.total_deliverables, m.completion_pct() * 100.0),
                Style::default().fg(Color::Gray),
            ),
        ]),
    ];

    if !m.review_status.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  Review status: ", Style::default().fg(Color::Gray)),
            Span::styled(&m.review_status, Style::default().fg(theme::GREEN)),
        ]));
    }

    if m.is_active {
        lines.push(Line::from(Span::styled(
            "  ← Active milestone",
            Style::default().fg(Color::Yellow),
        )));
    }

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Detail ")),
        area,
    );
}
