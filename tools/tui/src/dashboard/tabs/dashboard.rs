use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::super::app::{App, LiveTestCount};
use crate::widgets::progress_bar::progress_bar;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // headline + PB progress
            Constraint::Length(7),  // card health + summary lines
            Constraint::Min(0),    // path to alpha + workstreams
        ])
        .split(area);

    render_headline(f, rows[0], app);
    render_middle(f, rows[1], app);
    render_bottom(f, rows[2], app);
}

// ─── Row 1: Headline + PB Progress ─────────────────────────────────────────

fn render_headline(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: project title + key stats
    let cs = &app.data.current_state;
    let test_str = match &app.live_test_count {
        LiveTestCount::Loading => "...".to_string(),
        LiveTestCount::Done(n) => n.to_string(),
    };
    let headline = vec![
        Line::from(vec![Span::styled(
            "MTG Commander Rules Engine",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("Active: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &cs.active_milestone,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Tests: ", Style::default().fg(Color::Gray)),
            Span::styled(
                test_str,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Updated: ", Style::default().fg(Color::Gray)),
            Span::raw(&cs.last_updated),
        ]),
    ];

    f.render_widget(
        Paragraph::new(Text::from(headline))
            .block(Block::default().borders(Borders::ALL).title(" Project ")),
        cols[0],
    );

    // Right: PB progress
    let p = &app.data.progress;
    let batches_done = p
        .primitive_batches
        .iter()
        .filter(|b| b.status == "done")
        .count();
    let batches_total = p.primitive_batches.len();
    let ratio = if batches_total > 0 {
        batches_done as f64 / batches_total as f64
    } else {
        0.0
    };

    let inner_width = cols[1].width.saturating_sub(4);
    let bar = progress_bar(
        ratio,
        inner_width,
        &format!("{}/{} ({:.0}%)", batches_done, batches_total, ratio * 100.0),
        Color::Cyan,
    );

    // Worker status line
    let worker_line = if let Some(ws) = &app.data.worker_status {
        Line::from(vec![
            Span::styled("Worker: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &ws.batch,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(&ws.phase, Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(
                format!("({})", ws.title),
                Style::default().fg(Color::DarkGray),
            ),
        ])
    } else {
        Line::from(Span::styled(
            "No active worker",
            Style::default().fg(Color::DarkGray),
        ))
    };

    let pb_lines = vec![bar, worker_line];

    f.render_widget(
        Paragraph::new(Text::from(pb_lines)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Primitive Batches "),
        ),
        cols[1],
    );
}

// ─── Row 2: Card Health + Summary Lines ─────────────────────────────────────

fn render_middle(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    render_card_health(f, cols[0], app);
    render_summary_lines(f, cols[1], app);
}

fn render_card_health(f: &mut Frame, area: Rect, app: &App) {
    let h = &app.data.progress.card_health;
    let total = h.total_universe.max(1);
    let authored = h.total_authored;
    let ratio = authored as f64 / total as f64;

    let inner_width = area.width.saturating_sub(4);
    let bar = progress_bar(
        ratio,
        inner_width,
        &format!("{}/{} ({:.0}%)", authored, total, ratio * 100.0),
        Color::Cyan,
    );

    let todo_pct = if authored > 0 {
        h.has_todos * 100 / authored
    } else {
        0
    };

    let lines = vec![
        bar,
        Line::from(vec![
            Span::styled(
                format!(" OK {:>4}", h.fully_implemented),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("  Partial {:>4}", h.partial),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("  Strip {:>3}", h.stripped),
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" Van {:>4}", h.vanilla),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("  Left {:>4}", h.not_authored),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  "),
            Span::styled(
                format!("TODO: {}%", todo_pct),
                if todo_pct > 40 {
                    Style::default().fg(Color::Red)
                } else if todo_pct > 20 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(Text::from(lines)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Card Health (live) "),
        ),
        area,
    );
}

fn render_summary_lines(f: &mut Frame, area: Rect, app: &App) {
    let fmt_loc = |n: u32| -> String {
        if n >= 1000 {
            format!("{}k", n / 1000)
        } else {
            n.to_string()
        }
    };

    let r = &app.data.reviews;
    let cc = &app.data.corner_cases;
    let abilities_validated: u32 = app.data.abilities.summary.iter().map(|s| s.validated).sum();
    let abilities_total: u32 = app.data.abilities.summary.iter().map(|s| s.total).sum();

    let lines = vec![
        Line::from(vec![
            Span::styled("Abilities: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}/{} validated", abilities_validated, abilities_total),
                Style::default().fg(Color::Green),
            ),
            Span::raw("   "),
            Span::styled("Corner Cases: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}/{}", cc.covered, cc.total),
                Style::default().fg(if cc.gap > 0 {
                    Color::Yellow
                } else {
                    Color::Green
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Reviews: ", Style::default().fg(Color::Gray)),
            Span::styled("H:", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", r.high_open),
                if r.high_open > 0 {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
            Span::raw("/"),
            Span::styled(
                format!("{}", r.high_closed),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  "),
            Span::styled("M:", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", r.medium_open),
                if r.medium_open > 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
            Span::raw("/"),
            Span::styled(
                format!("{}", r.medium_closed),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  "),
            Span::styled("L:", Style::default().fg(Color::White)),
            Span::styled(format!("{}", r.low_open), Style::default().fg(Color::Gray)),
            Span::raw("/"),
            Span::styled(
                format!("{}", r.low_closed),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("Scripts: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!(
                    "{} total, {} approved",
                    app.data.scripts.total, app.data.scripts.approved
                ),
                Style::default().fg(Color::Green),
            ),
            Span::raw("   "),
            Span::styled("Engine: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} src + {} test LOC", fmt_loc(r.engine_loc), fmt_loc(r.test_loc)),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(Text::from(lines)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Summary "),
        ),
        area,
    );
}

// ─── Row 3: Path to Alpha + Workstreams ─────────────────────────────────────

fn render_bottom(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    render_path_to_alpha(f, cols[0], app);
    render_workstreams(f, cols[1], app);
}

fn render_path_to_alpha(f: &mut Frame, area: Rect, app: &App) {
    let milestones = &app.data.progress.path_to_alpha;
    let mut lines: Vec<Line> = vec![];

    for m in milestones {
        let color = match m.status.as_str() {
            "done" => Color::Green,
            "active" => Color::Yellow,
            "blocked" => Color::Red,
            _ => Color::DarkGray,
        };
        let icon = match m.status.as_str() {
            "done" => "v",
            "active" => ">",
            "blocked" => "x",
            _ => "-",
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(color)),
            Span::styled(
                truncate(&m.name, (area.width as usize).saturating_sub(8)),
                Style::default().fg(color),
            ),
            Span::raw("  "),
            Span::styled(
                truncate(&m.deliverable, 30),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Path to Alpha ")),
        area,
    );
}

fn render_workstreams(f: &mut Frame, area: Rect, app: &App) {
    let ws = &app.data.progress.workstreams;
    let mut lines: Vec<Line> = vec![];

    for w in ws {
        let status_color = match w.status.as_str() {
            "done" => Color::Green,
            "active" => Color::Yellow,
            "stalled" => Color::Red,
            "not-started" => Color::DarkGray,
            "retired" => Color::DarkGray,
            _ => Color::White,
        };
        let icon = match w.status.as_str() {
            "done" => "v",
            "active" => ">",
            "stalled" => "!",
            "retired" => "x",
            _ => "-",
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(status_color)),
            Span::styled(
                format!("{} ", w.number),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                truncate(&w.name, 20),
                Style::default().fg(status_color),
            ),
        ]));
    }

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Workstreams ")),
        area,
    );
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}..", &s[..max.saturating_sub(2)])
    }
}
