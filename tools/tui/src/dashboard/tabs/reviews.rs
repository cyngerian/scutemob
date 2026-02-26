use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::theme;
use super::super::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_open_issues(f, chunks[0], app);
    render_stats(f, chunks[1], app);
}

fn render_open_issues(f: &mut Frame, area: Rect, app: &App) {
    let r = &app.data.reviews;

    let mut lines: Vec<Line> = vec![];

    // HIGH
    lines.push(Line::from(vec![
        Span::styled("HIGH", Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)),
        Span::raw(format!("  {} open / {} closed", r.high_open, r.high_closed)),
    ]));
    if r.high_open == 0 {
        lines.push(Line::from(Span::styled("  ✓ All resolved", Style::default().fg(theme::GREEN))));
    }
    lines.push(Line::from(""));

    // MEDIUM
    lines.push(Line::from(vec![
        Span::styled("MEDIUM", Style::default().fg(theme::GOLD).add_modifier(Modifier::BOLD)),
        Span::raw(format!("  {} open / {} closed", r.medium_open, r.medium_closed)),
    ]));
    if r.medium_open == 0 {
        lines.push(Line::from(Span::styled("  ✓ All resolved", Style::default().fg(theme::GREEN))));
    } else {
        lines.push(Line::from(Span::styled(
            "  MR-M7-09, MR-M7-12 (deferred to M10+)",
            Style::default().fg(Color::DarkGray),
        )));
    }
    lines.push(Line::from(""));

    // LOW
    lines.push(Line::from(vec![
        Span::styled("LOW", Style::default().fg(theme::ARTIFACT).add_modifier(Modifier::BOLD)),
        Span::raw(format!("  {} open / {} closed", r.low_open, r.low_closed)),
    ]));
    if r.low_open > 0 {
        lines.push(Line::from(Span::styled(
            "  Deferred — address opportunistically",
            Style::default().fg(Color::DarkGray),
        )));
    }
    lines.push(Line::from(""));

    // INFO
    lines.push(Line::from(vec![
        Span::styled("INFO", Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD)),
        Span::raw(format!("  {} total", r.info)),
    ]));

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Issue Summary ")),
        area,
    );
}

fn render_stats(f: &mut Frame, area: Rect, app: &App) {
    let r = &app.data.reviews;

    let total_open = r.high_open + r.medium_open + r.low_open;
    let total_closed = r.high_closed + r.medium_closed + r.low_closed;

    let lines = vec![
        Line::from(vec![
            Span::styled("Total issues: ", Style::default().fg(Color::Gray)),
            Span::styled(r.total_issues.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Open:   ", Style::default().fg(Color::Gray)),
            Span::styled(
                total_open.to_string(),
                if total_open > 0 { Style::default().fg(theme::GOLD) } else { Style::default().fg(theme::GREEN) },
            ),
        ]),
        Line::from(vec![
            Span::styled("Closed: ", Style::default().fg(Color::Gray)),
            Span::styled(total_closed.to_string(), Style::default().fg(theme::GREEN)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Milestones reviewed: ", Style::default().fg(Color::Gray)),
            Span::styled(r.milestones_reviewed.to_string(), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Engine source: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("~{} LOC", r.engine_loc), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Engine tests:  ", Style::default().fg(Color::Gray)),
            Span::styled(format!("~{} LOC", r.test_loc), Style::default().fg(Color::White)),
        ]),
    ];

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Statistics ")),
        area,
    );
}
