use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::dashboard::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // pipeline funnel + card health
            Constraint::Length(12), // primitive batches (scrollable region)
            Constraint::Min(0),     // review backlog + workstreams + deferred
        ])
        .split(area);

    // Row 1: funnel + card health side by side
    let row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    render_pipeline_funnel(f, row1[0], app);
    render_card_health(f, row1[1], app);

    // Row 2: primitive batches
    render_primitive_batches(f, chunks[1], app);

    // Row 3: review backlog + workstreams + path to alpha
    let row3 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Percentage(30),
            Constraint::Percentage(25),
        ])
        .split(chunks[2]);

    render_review_backlog(f, row3[0], app);
    render_workstreams(f, row3[1], app);
    render_path_to_alpha(f, row3[2], app);
}

fn render_pipeline_funnel(f: &mut Frame, area: Rect, app: &App) {
    let p = &app.data.progress;
    let batches_done = p
        .primitive_batches
        .iter()
        .filter(|b| b.status == "done")
        .count();
    let batches_total = p.primitive_batches.len();
    let review_done = p
        .review_backlog
        .iter()
        .filter(|r| r.review_status == "clean" || r.review_status == "fixed")
        .count();

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Primitives: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}/{}", batches_done, batches_total),
                Style::default()
                    .fg(if batches_done == batches_total {
                        Color::Green
                    } else {
                        Color::Yellow
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Reviews: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}/{}", review_done, p.review_backlog.len()),
                Style::default()
                    .fg(if review_done == p.review_backlog.len() {
                        Color::Green
                    } else {
                        Color::Cyan
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Tests: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", app.data.current_state.test_count),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  "),
            Span::styled("Abilities: ", Style::default().fg(Color::Gray)),
            Span::raw("194/204"),
            Span::raw("  "),
            Span::styled("Corner: ", Style::default().fg(Color::Gray)),
            Span::raw("32/36"),
        ]),
    ];

    // Progress bar for primitives
    let pct = if batches_total > 0 {
        batches_done * 100 / batches_total
    } else {
        0
    };
    let bar_width = (area.width as usize).saturating_sub(4).min(40);
    let filled = bar_width * pct / 100;
    let bar: String = format!(
        "[{}{}] {}%",
        "#".repeat(filled),
        "-".repeat(bar_width - filled),
        pct
    );
    lines.push(Line::from(vec![
        Span::styled("PB ", Style::default().fg(Color::Gray)),
        Span::styled(bar, Style::default().fg(Color::Cyan)),
    ]));

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Pipeline ")),
        area,
    );
}

fn render_card_health(f: &mut Frame, area: Rect, app: &App) {
    let h = &app.data.progress.card_health;
    let total = h.total_universe.max(1);

    let lines = vec![
        Line::from(vec![
            Span::styled("Universe: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", h.total_universe),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Authored: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", h.total_authored),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                format!(" ({}%)", h.total_authored * 100 / total),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                " OK  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{:<6}", h.complete)),
            Span::styled(
                " TODO ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{:<6}", h.has_todos)),
            Span::styled(
                " BAD  ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{:<6}", h.wrong_state)),
        ]),
        Line::from(vec![
            Span::styled("Not authored: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", h.not_authored),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(Text::from(lines)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Card Health "),
        ),
        area,
    );
}

fn focused_block<'a>(title: String, is_focused: bool) -> Block<'a> {
    let block = Block::default().borders(Borders::ALL).title(title);
    if is_focused {
        block.border_style(Style::default().fg(Color::Cyan))
    } else {
        block
    }
}

fn render_primitive_batches(f: &mut Frame, area: Rect, app: &App) {
    let batches = &app.data.progress.primitive_batches;
    let scroll = app.progress_scroll as usize;
    let max_scroll = batches.len().saturating_sub(1);
    let scroll = scroll.min(max_scroll);
    let visible_rows = area.height.saturating_sub(3) as usize; // borders + header

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
            format!("{:<35}", "Title"),
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

    for batch in batches.iter().skip(scroll).take(visible_rows) {
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

        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<8}", batch.batch),
                Style::default().fg(Color::White),
            ),
            Span::raw(format!("{:<35}", truncate(&batch.title, 34))),
            Span::styled(
                format!("{:<8}", status_icon),
                Style::default().fg(status_color),
            ),
            Span::raw(format!("{:<7}", batch.cards_fixed)),
            Span::raw(format!("{:<7}", batch.cards_remaining)),
            Span::styled(batch.review.to_string(), Style::default().fg(review_color)),
        ]));
    }

    let is_focused = app.progress_focus == 0;
    let title = format!(" Primitive Batches ({} total) ", batches.len());
    f.render_widget(
        Paragraph::new(Text::from(lines)).block(focused_block(title, is_focused)),
        area,
    );
}

fn render_review_backlog(f: &mut Frame, area: Rect, app: &App) {
    let backlog = &app.data.progress.review_backlog;
    let done_count = backlog
        .iter()
        .filter(|r| r.review_status == "clean" || r.review_status == "fixed")
        .count();
    let scroll = app.progress_backlog_scroll as usize;
    let max_scroll = backlog.len().saturating_sub(1);
    let scroll = scroll.min(max_scroll);

    let mut lines: Vec<Line> = vec![];

    // Header
    lines.push(Line::from(vec![
        Span::styled(format!("{:<3}", "#"), Style::default().fg(Color::Gray)),
        Span::styled(format!("{:<8}", "Batch"), Style::default().fg(Color::Gray)),
        Span::styled(format!("{:<22}", "Title"), Style::default().fg(Color::Gray)),
        Span::styled(format!("{:<6}", "Cards"), Style::default().fg(Color::Gray)),
        Span::styled("Status", Style::default().fg(Color::Gray)),
    ]));

    let visible = area.height.saturating_sub(3) as usize;
    for entry in backlog.iter().skip(scroll).take(visible) {
        let status_style = match entry.review_status.as_str() {
            "clean" => Style::default().fg(Color::Green),
            "fixed" => Style::default().fg(Color::Cyan),
            "in-review" => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            "needs-fix" | "fixing" => Style::default().fg(Color::Red),
            "pending" => Style::default().fg(Color::DarkGray),
            _ => Style::default(),
        };

        lines.push(Line::from(vec![
            Span::raw(format!("{:<3}", entry.number)),
            Span::raw(format!("{:<8}", entry.batch)),
            Span::raw(format!("{:<22}", truncate(&entry.title, 21))),
            Span::raw(format!("{:<6}", entry.cards_fixed)),
            Span::styled(entry.review_status.to_string(), status_style),
        ]));
    }

    let is_focused = app.progress_focus == 1;
    let title = format!(" Review Backlog ({}/{}) ", done_count, backlog.len());
    f.render_widget(
        Paragraph::new(Text::from(lines)).block(focused_block(title, is_focused)),
        area,
    );
}

fn render_workstreams(f: &mut Frame, area: Rect, app: &App) {
    let ws = &app.data.progress.workstreams;
    let scroll = app.progress_workstream_scroll as usize;
    let max_scroll = ws.len().saturating_sub(1);
    let scroll = scroll.min(max_scroll);
    let visible = area.height.saturating_sub(2) as usize;
    let mut lines: Vec<Line> = vec![];

    for w in ws.iter().skip(scroll).take(visible) {
        let status_color = match w.status.as_str() {
            "done" => Color::Green,
            "active" => Color::Yellow,
            "stalled" => Color::Red,
            "partial" => Color::Cyan,
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
            Span::raw(truncate(&w.name, 18)),
        ]));
    }

    let is_focused = app.progress_focus == 2;
    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(focused_block(" Workstreams ".to_string(), is_focused)),
        area,
    );
}

fn render_path_to_alpha(f: &mut Frame, area: Rect, app: &App) {
    let milestones = &app.data.progress.path_to_alpha;
    let scroll = app.progress_alpha_scroll as usize;
    let max_scroll = milestones.len().saturating_sub(1);
    let scroll = scroll.min(max_scroll);
    let visible = area.height.saturating_sub(2) as usize;
    let mut lines: Vec<Line> = vec![];

    for m in milestones.iter().skip(scroll).take(visible) {
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
                truncate(&m.name, (area.width as usize).saturating_sub(5)),
                Style::default().fg(color),
            ),
        ]));
    }

    let is_focused = app.progress_focus == 3;
    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(focused_block(" Path to Alpha ".to_string(), is_focused)),
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
