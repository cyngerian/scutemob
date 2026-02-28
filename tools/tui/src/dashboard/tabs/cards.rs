use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use super::super::app::App;
use crate::theme;

pub fn render(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // summary
            Constraint::Min(0),    // table
            Constraint::Length(5), // detail (3 content lines + 2 borders)
        ])
        .split(area);

    render_summary(f, chunks[0], app);
    render_table(f, chunks[1], app);
    render_detail(f, chunks[2], app);
}

fn render_summary(f: &mut Frame, area: Rect, app: &App) {
    let c = &app.data.cards;

    let filter_hint = match app.cards_filter.as_str() {
        "ready" => " [r:ready  b:blocked  d:deferred  a:all]",
        "blocked" => " [r:ready  b:blocked  d:deferred  a:all]",
        "deferred" => " [r:ready  b:blocked  d:deferred  a:all]",
        _ => " [r:ready  b:blocked  d:deferred]",
    };

    let line = Line::from(vec![
        Span::styled("Ready: ", Style::default().fg(Color::Gray)),
        Span::styled(c.ready.to_string(), Style::default().fg(theme::GREEN)),
        Span::raw("   "),
        Span::styled("Blocked: ", Style::default().fg(Color::Gray)),
        Span::styled(
            c.blocked.to_string(),
            Style::default().fg(theme::RED),
        ),
        Span::raw("   "),
        Span::styled("Deferred: ", Style::default().fg(Color::Gray)),
        Span::styled(
            c.deferred.to_string(),
            Style::default().fg(theme::ARTIFACT),
        ),
        Span::raw("   "),
        Span::styled("Unknown: ", Style::default().fg(Color::Gray)),
        Span::styled(
            c.unknown.to_string(),
            if c.unknown > 0 {
                Style::default().fg(theme::GOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::styled(filter_hint, Style::default().fg(Color::DarkGray)),
    ]);

    f.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::ALL).title(" Summary ")),
        area,
    );
}

fn render_table(f: &mut Frame, area: Rect, app: &mut App) {
    let filter = &app.cards_filter;
    let entries: Vec<&crate::dashboard::data::CardWorklistEntry> = app
        .data
        .cards
        .entries
        .iter()
        .filter(|e| filter == "all" || e.status == *filter)
        .collect();

    let header = Row::new(vec![
        Cell::from("").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Dks").style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Gray),
        ),
        Cell::from("Types").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Keywords").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Blockers").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().fg(Color::White).bg(Color::DarkGray));

    let rows: Vec<Row> = entries
        .iter()
        .map(|e| {
            let (symbol, sym_style) = match e.status.as_str() {
                "ready" => ("\u{2713}", Style::default().fg(theme::GREEN)),    // ✓
                "blocked" => ("\u{2717}", Style::default().fg(theme::RED)),     // ✗
                "deferred" => ("\u{25CC}", Style::default().fg(theme::ARTIFACT)), // ◌
                _ => ("?", Style::default().fg(theme::GOLD)),
            };

            let name_style = match e.status.as_str() {
                "blocked" => Style::default().fg(theme::RED),
                "deferred" => Style::default().fg(theme::ARTIFACT),
                _ => Style::default().fg(Color::White),
            };

            let types_str = e.types.join(", ");
            let kw_str = e.keywords.join(", ");
            let blockers_str = e.blocking_keywords.join(", ");

            Row::new(vec![
                Cell::from(Span::styled(symbol, sym_style)),
                Cell::from(Span::styled(e.name.clone(), name_style)),
                Cell::from(Span::styled(
                    format!("{:>2}", e.appears_in_decks),
                    Style::default().fg(Color::Gray),
                )),
                Cell::from(Span::styled(types_str, Style::default().fg(Color::DarkGray))),
                Cell::from(Span::styled(kw_str, Style::default().fg(Color::Gray))),
                Cell::from(Span::styled(blockers_str, Style::default().fg(theme::RED))),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(2),  // symbol
        Constraint::Min(20),    // name
        Constraint::Length(3),  // decks
        Constraint::Length(14), // types
        Constraint::Length(18), // keywords
        Constraint::Length(18), // blockers
    ];

    let visible = entries.len();
    let filter_label = match filter.as_str() {
        "ready" => "ready",
        "blocked" => "blocked",
        "deferred" => "deferred",
        _ => "all",
    };
    let title = format!(" Cards — {} ({}) — j/k scroll ", filter_label, visible);

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("\u{2192} "); // →

    f.render_stateful_widget(table, area, &mut app.cards_table_state);
}

fn render_detail(f: &mut Frame, area: Rect, app: &App) {
    let filter = &app.cards_filter;
    let entries: Vec<&crate::dashboard::data::CardWorklistEntry> = app
        .data
        .cards
        .entries
        .iter()
        .filter(|e| filter == "all" || e.status == *filter)
        .collect();

    let sel = app.cards_table_state.selected().unwrap_or(0);
    let text = if let Some(entry) = entries.get(sel) {
        let (sym, sym_style) = match entry.status.as_str() {
            "ready" => ("\u{2713}", Style::default().fg(theme::GREEN)),
            "blocked" => ("\u{2717}", Style::default().fg(theme::RED)),
            "deferred" => ("\u{25CC}", Style::default().fg(theme::ARTIFACT)),
            _ => ("?", Style::default().fg(theme::GOLD)),
        };

        // Line 1: symbol + name + types + deck count
        let line1 = Line::from(vec![
            Span::styled(format!("{} ", sym), sym_style),
            Span::styled(
                &entry.name,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  [{}]", entry.types.join(", ")),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("  in {} deck(s)", entry.appears_in_decks),
                Style::default().fg(Color::Gray),
            ),
        ]);

        // Line 2: keyword statuses
        let kw_spans: Vec<Span> = if entry.keyword_statuses.is_empty() {
            vec![Span::styled(
                "  No keywords",
                Style::default().fg(Color::DarkGray),
            )]
        } else {
            let mut spans = vec![Span::styled("  Keywords: ", Style::default().fg(Color::Gray))];
            for (i, (kw, st)) in entry.keyword_statuses.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw(", "));
                }
                let color = if st.contains("validated") || st == "ready" {
                    theme::GREEN
                } else if st.contains("deferred") {
                    theme::ARTIFACT
                } else if st.contains("none") {
                    theme::RED
                } else {
                    theme::GOLD
                };
                spans.push(Span::styled(
                    format!("{} ({})", kw, st),
                    Style::default().fg(color),
                ));
            }
            spans
        };
        let line2 = Line::from(kw_spans);

        // Line 3: blockers (if any)
        let line3 = if !entry.blocking_keywords.is_empty() {
            Line::from(vec![
                Span::styled("  Blocked by: ", Style::default().fg(theme::RED)),
                Span::styled(
                    entry.blocking_keywords.join(", "),
                    Style::default().fg(theme::RED),
                ),
            ])
        } else {
            Line::from(Span::raw(""))
        };

        Text::from(vec![line1, line2, line3])
    } else {
        Text::from(Line::from(Span::styled(
            "\u{2014}",
            Style::default().fg(Color::DarkGray),
        )))
    };

    f.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Detail "),
        ),
        area,
    );
}
