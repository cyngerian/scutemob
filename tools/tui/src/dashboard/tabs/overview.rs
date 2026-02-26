use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{theme, widgets::progress_bar::progress_bar};
use super::super::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    // Outer: 3 rows vertically
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // headline + milestone gauge
            Constraint::Length(8),  // ability coverage + corner cases
            Constraint::Min(5),     // reviews + scripts + engine size
        ])
        .split(area);

    render_headline(f, rows[0], app);
    render_middle_row(f, rows[1], app);
    render_bottom_row(f, rows[2], app);
}

// ─── Row 1: Headline + Milestone Gauge ──────────────────────────────────────

fn render_headline(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: project title + stats
    let cs = &app.data.current_state;
    let headline = vec![
        Line::from(vec![
            Span::styled("MTG Commander Rules Engine", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Active: ", Style::default().fg(Color::Gray)),
            Span::styled(&cs.active_milestone, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("   "),
            Span::styled("Tests: ", Style::default().fg(Color::Gray)),
            Span::styled(cs.test_count.to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("   "),
            Span::styled("Scripts: ", Style::default().fg(Color::Gray)),
            Span::styled(cs.script_count.to_string(), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Last Updated: ", Style::default().fg(Color::Gray)),
            Span::raw(&cs.last_updated),
        ]),
    ];

    f.render_widget(
        Paragraph::new(Text::from(headline))
            .block(Block::default().borders(Borders::ALL).title(" Project ")),
        cols[0],
    );

    // Right: milestone progress
    let milestones = &app.data.milestones;
    let total = milestones.len();
    let done = milestones.iter().filter(|m| m.completion_pct() >= 1.0).count();
    let ratio = if total > 0 { done as f64 / total as f64 } else { 0.0 };

    let gauge_width = cols[1].width.saturating_sub(4);
    let trail: String = milestones
        .iter()
        .map(|m| {
            if m.is_active {
                format!("[{}]", m.id)
            } else if m.completion_pct() >= 1.0 {
                format!("{}✓", m.id)
            } else {
                m.id.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let bar_line = progress_bar(ratio, gauge_width, &format!("{}/{} ({:.0}%)", done, total, ratio * 100.0), Color::Green);

    let milestone_lines = vec![
        bar_line,
        Line::from(Span::styled(trail, Style::default().fg(Color::Gray))),
    ];

    f.render_widget(
        Paragraph::new(Text::from(milestone_lines))
            .block(Block::default().borders(Borders::ALL).title(" Milestone Progress ")),
        cols[1],
    );
}

// ─── Row 2: Ability Coverage + Corner Cases ──────────────────────────────────

fn render_middle_row(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    render_ability_summary(f, cols[0], app);
    render_corner_cases_summary(f, cols[1], app);
}

fn render_ability_summary(f: &mut Frame, area: Rect, app: &App) {
    let inner_width = area.width.saturating_sub(4);
    // Fixed label width so all bars align: "P1: 42/42 validated" = 20 chars max
    let label_width = 20u16;
    let bar_width = inner_width.saturating_sub(label_width + 1);
    let mut lines: Vec<Line> = vec![];

    for row in &app.data.abilities.summary {
        if row.priority.starts_with("Total") || row.priority.is_empty() { continue; }
        let ratio = if row.total > 0 { row.validated as f64 / row.total as f64 } else { 0.0 };
        let label = format!("{}: {:>2}/{:<2} validated", row.priority, row.validated, row.total);
        let filled = ((ratio.clamp(0.0, 1.0) * bar_width as f64) as u16).min(bar_width);
        let empty = bar_width - filled;
        lines.push(Line::from(vec![
            Span::styled(format!("{:<20}", label), Style::default().fg(Color::White)),
            Span::raw(" "),
            Span::styled("█".repeat(filled as usize), Style::default().fg(theme::GREEN)),
            Span::styled("░".repeat(empty as usize), Style::default().fg(Color::DarkGray)),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from("No ability data"));
    }

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Ability Coverage ")),
        area,
    );
}

fn render_corner_cases_summary(f: &mut Frame, area: Rect, app: &App) {
    let cc = &app.data.corner_cases;
    let total = cc.total();
    let covered_pct = if total > 0 { cc.covered as f64 / total as f64 } else { 0.0 };

    let inner_width = area.width.saturating_sub(4);
    let bar = progress_bar(covered_pct, inner_width, &format!("{}/{}", cc.covered, total), Color::Green);

    let lines = vec![
        bar,
        Line::from(vec![
            Span::styled(format!("Covered: {:>2}  ({:.0}%)", cc.covered, covered_pct * 100.0), Style::default().fg(theme::GREEN)),
            Span::raw("  "),
            Span::styled(format!("Gap: {:>2}", cc.gap), Style::default().fg(theme::RED)),
        ]),
        Line::from(vec![
            Span::styled(format!("Partial: {:>2}", cc.partial), Style::default().fg(theme::GOLD)),
            Span::raw("  "),
            Span::styled(format!("Deferred: {:>2}", cc.deferred), Style::default().fg(theme::ARTIFACT)),
        ]),
    ];

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Corner Cases ")),
        area,
    );
}

// ─── Row 3: Reviews + Scripts + Engine Size ──────────────────────────────────

fn render_bottom_row(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30),  // Code Reviews (compact)
            Constraint::Min(20),    // Scripts (fill remaining)
            Constraint::Length(24), // Engine Size (compact)
        ])
        .split(area);

    render_reviews_summary(f, cols[0], app);
    render_scripts(f, cols[1], app);
    render_engine_size(f, cols[2], app);
}

fn render_reviews_summary(f: &mut Frame, area: Rect, app: &App) {
    let r = &app.data.reviews;
    let lines = vec![
        Line::from(vec![
            Span::styled("HIGH:   ", Style::default().fg(Color::White)),
            Span::styled(format!("{} open", r.high_open), if r.high_open > 0 { Style::default().fg(theme::RED) } else { Style::default().fg(theme::GREEN) }),
            Span::raw("  "),
            Span::styled(format!("{} closed", r.high_closed), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("MEDIUM: ", Style::default().fg(Color::White)),
            Span::styled(format!("{} open", r.medium_open), if r.medium_open > 0 { Style::default().fg(theme::GOLD) } else { Style::default().fg(theme::GREEN) }),
            Span::raw("  "),
            Span::styled(format!("{} closed", r.medium_closed), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("LOW:    ", Style::default().fg(Color::White)),
            Span::styled(format!("{} open", r.low_open), Style::default().fg(Color::Gray)),
            Span::raw("  "),
            Span::styled(format!("{} closed", r.low_closed), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(format!("{} milestones reviewed", r.milestones_reviewed), Style::default().fg(Color::DarkGray)),
        ]),
    ];

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Code Reviews ")),
        area,
    );
}

fn render_scripts(f: &mut Frame, area: Rect, app: &App) {
    let scripts = &app.data.scripts;
    let inner_width = area.width.saturating_sub(4);
    let max_count = scripts.by_directory.iter().map(|(_, c)| *c).max().unwrap_or(1);

    // Compute max dir name length for alignment
    let name_width = scripts.by_directory.iter()
        .map(|(d, _)| d.len())
        .max()
        .unwrap_or(8)
        .max(8) + 1; // +1 for padding
    let count_width = 4u16; // " XX"
    let bar_width = inner_width.saturating_sub(name_width as u16 + count_width);

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            format!("Total: {}", scripts.total),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
    ];

    for (dir, count) in &scripts.by_directory {
        let ratio = if max_count > 0 { *count as f64 / max_count as f64 } else { 0.0 };
        let filled = ((ratio * bar_width as f64) as u16).min(bar_width);
        lines.push(Line::from(vec![
            Span::styled(format!("{:<w$}", dir, w = name_width), Style::default().fg(Color::Gray)),
            Span::styled("█".repeat(filled as usize), Style::default().fg(theme::BLUE)),
            Span::styled("░".repeat((bar_width - filled) as usize), Style::default().fg(Color::DarkGray)),
            Span::styled(format!(" {:>2}", count), Style::default().fg(Color::White)),
        ]));
    }

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Scripts ")),
        area,
    );
}

fn render_engine_size(f: &mut Frame, area: Rect, app: &App) {
    let r = &app.data.reviews;
    let lines = vec![
        Line::from(vec![
            Span::styled("Source: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("~{:>6} LOC", r.engine_loc), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Tests:  ", Style::default().fg(Color::Gray)),
            Span::styled(format!("~{:>6} LOC", r.test_loc), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Scripts: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} JSON", app.data.scripts.total),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Engine Size ")),
        area,
    );
}
