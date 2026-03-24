use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::super::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // worker status + summary
            Constraint::Min(0),   // batch table
        ])
        .split(area);

    render_worker_status(f, chunks[0], app);
    render_batches(f, chunks[1], app);
}

fn render_worker_status(f: &mut Frame, area: Rect, app: &App) {
    let p = &app.data.progress;
    let done = p
        .primitive_batches
        .iter()
        .filter(|b| b.status == "done")
        .count();
    let planned = p
        .primitive_batches
        .iter()
        .filter(|b| b.status == "planned")
        .count();
    let total = p.primitive_batches.len();

    let mut lines = vec![Line::from(vec![
        Span::styled("Done: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", done),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Planned: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", planned),
            if planned > 0 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            },
        ),
        Span::raw("  "),
        Span::styled("Total: ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", total), Style::default().fg(Color::White)),
    ])];

    if let Some(ws) = &app.data.worker_status {
        lines.push(Line::from(vec![
            Span::styled("Active: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} ", ws.batch),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("[{}] ", ws.phase),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(&ws.title, Style::default().fg(Color::White)),
            if !ws.started.is_empty() {
                Span::styled(
                    format!("  (since {})", ws.started),
                    Style::default().fg(Color::DarkGray),
                )
            } else {
                Span::raw("")
            },
        ]));
    }

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Pipeline Status ")),
        area,
    );
}

fn render_batches(f: &mut Frame, area: Rect, app: &App) {
    let batches = &app.data.progress.primitive_batches;
    let scroll = app.pipeline_scroll as usize;
    let visible_rows = area.height.saturating_sub(3) as usize;

    // Reverse sort: planned/active at top, done at bottom
    let mut sorted: Vec<(usize, &super::super::data::PrimitiveBatch)> =
        batches.iter().enumerate().collect();
    sorted.sort_by(|(_, a), (_, b)| {
        let order = |s: &str| -> u8 {
            match s {
                "active" => 0,
                "planned" => 1,
                "done" => 2,
                _ => 3,
            }
        };
        order(&a.status)
            .cmp(&order(&b.status))
            .then(a.batch.cmp(&b.batch))
    });

    let max_scroll = sorted.len().saturating_sub(1);
    let scroll = scroll.min(max_scroll);

    let mut lines: Vec<Line> = vec![];

    // Header
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<8}", "Batch"),
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<40}", "Title"),
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<8}", "Status"),
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<7}", "Fixed"),
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<7}", "Left"),
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Review",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    for (_, batch) in sorted.iter().skip(scroll).take(visible_rows) {
        let status_color = match batch.status.as_str() {
            "done" => Color::Green,
            "active" => Color::Yellow,
            "planned" => Color::DarkGray,
            _ => Color::White,
        };
        let review_color = match batch.review.as_str() {
            "clean" => Color::Green,
            "fixed" => Color::Cyan,
            "none" => Color::Yellow,
            _ => Color::DarkGray,
        };
        let status_icon = match batch.status.as_str() {
            "done" => "done",
            "active" => ">>",
            "planned" => "  --",
            _ => &batch.status,
        };

        // Dim done rows
        let name_style = if batch.status == "done" {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<8}", batch.batch),
                if batch.status == "done" {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Span::styled(format!("{:<40}", truncate(&batch.title, 39)), name_style),
            Span::styled(format!("{:<8}", status_icon), Style::default().fg(status_color)),
            Span::styled(
                format!("{:<7}", batch.cards_fixed),
                if batch.status == "done" {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Span::styled(
                format!("{:<7}", batch.cards_remaining),
                if batch.cards_remaining > 0 && batch.status != "done" {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Span::styled(batch.review.to_string(), Style::default().fg(review_color)),
        ]));
    }

    let title = format!(
        " Primitive Batches ({} total) — j/k scroll ",
        batches.len()
    );
    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(title)),
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
