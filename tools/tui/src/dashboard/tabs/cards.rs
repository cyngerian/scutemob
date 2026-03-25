use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use super::super::app::App;
use super::super::data::LiveCardEntry;

pub fn render(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // summary
            Constraint::Min(0),    // table
            Constraint::Min(8),    // detail
        ])
        .split(area);

    render_summary(f, chunks[0], app);
    render_table(f, chunks[1], app);
    render_detail(f, chunks[2], app);
}

fn render_summary(f: &mut Frame, area: Rect, app: &App) {
    let cards = &app.data.live_cards;
    let ok = cards.iter().filter(|c| c.status == "ok").count();
    let partial = cards.iter().filter(|c| c.status == "partial").count();
    let stripped = cards.iter().filter(|c| c.status == "stripped").count();
    let vanilla = cards.iter().filter(|c| c.status == "vanilla").count();
    let total = cards.len();

    let filter_hint = match app.cards_filter.as_str() {
        "all" => " [t:todo o:ok p:partial s:stripped a:all]",
        _ => " [t:todo o:ok p:partial s:stripped a:all]",
    };

    let line = Line::from(vec![
        Span::styled("Total: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", total),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(format!("OK: {}", ok), Style::default().fg(Color::Green)),
        Span::raw("   "),
        Span::styled(
            format!("Partial: {}", partial),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw("   "),
        Span::styled(
            format!("Stripped: {}", stripped),
            Style::default().fg(Color::Red),
        ),
        Span::raw("   "),
        Span::styled(
            format!("Vanilla: {}", vanilla),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(filter_hint, Style::default().fg(Color::DarkGray)),
    ]);

    f.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::ALL).title(" Summary ")),
        area,
    );
}

fn filtered_entries<'a>(cards: &'a [LiveCardEntry], filter: &str) -> Vec<&'a LiveCardEntry> {
    if filter == "all" {
        cards.iter().collect()
    } else if filter == "todo" {
        cards
            .iter()
            .filter(|c| c.status == "partial" || c.status == "stripped")
            .collect()
    } else {
        cards.iter().filter(|c| c.status == filter).collect()
    }
}

fn render_table(f: &mut Frame, area: Rect, app: &mut App) {
    let entries = filtered_entries(&app.data.live_cards, &app.cards_filter);

    let header = Row::new(vec![
        Cell::from("").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("File").style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Gray),
        ),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("TODOs").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().fg(Color::White).bg(Color::DarkGray));

    let rows: Vec<Row> = entries
        .iter()
        .map(|e| {
            let (symbol, sym_style) = match e.status.as_str() {
                "ok" => ("\u{2713}", Style::default().fg(Color::Green)),
                "partial" => ("\u{25D1}", Style::default().fg(Color::Yellow)),
                "stripped" => ("\u{2717}", Style::default().fg(Color::Red)),
                "vanilla" => ("\u{25CB}", Style::default().fg(Color::DarkGray)),
                _ => ("?", Style::default().fg(Color::White)),
            };

            let name_style = match e.status.as_str() {
                "ok" => Style::default().fg(Color::Green),
                "partial" => Style::default().fg(Color::Yellow),
                "stripped" => Style::default().fg(Color::Red),
                "vanilla" => Style::default().fg(Color::DarkGray),
                _ => Style::default().fg(Color::White),
            };

            let todo_count = e.todo_lines.len();
            let todo_str = if todo_count > 0 {
                format!("{}", todo_count)
            } else {
                String::new()
            };

            Row::new(vec![
                Cell::from(Span::styled(symbol, sym_style)),
                Cell::from(Span::styled(e.name.clone(), name_style)),
                Cell::from(Span::styled(
                    e.file_name.clone(),
                    Style::default().fg(Color::DarkGray),
                )),
                Cell::from(Span::styled(e.status.clone(), name_style)),
                Cell::from(Span::styled(
                    todo_str,
                    if todo_count > 0 {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                )),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(2),  // symbol
        Constraint::Min(25),    // name
        Constraint::Length(25), // file
        Constraint::Length(10), // status
        Constraint::Length(5),  // TODOs
    ];

    let visible = entries.len();
    let filter_label = match app.cards_filter.as_str() {
        "todo" => "has TODO",
        "ok" => "ok",
        "partial" => "partial",
        "stripped" => "stripped",
        "vanilla" => "vanilla",
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
        .highlight_symbol("\u{2192} ");

    f.render_stateful_widget(table, area, &mut app.cards_table_state);
}

fn render_detail(f: &mut Frame, area: Rect, app: &App) {
    let entries = filtered_entries(&app.data.live_cards, &app.cards_filter);
    let sel = app.cards_table_state.selected().unwrap_or(0);

    let (text, has_dsl) = if let Some(entry) = entries.get(sel) {
        // Try to get DSL source
        let dsl = app.data.card_dsl.get(&entry.name);

        if let Some(dsl_src) = dsl {
            let mut lines: Vec<Line> = Vec::new();
            for src_line in dsl_src.lines() {
                let line = if src_line.contains("TODO") {
                    Line::from(Span::styled(
                        src_line.to_string(),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ))
                } else if let Some(colon_pos) = src_line.find(':') {
                    let key_part = &src_line[..=colon_pos];
                    let val_part = &src_line[colon_pos + 1..];
                    Line::from(vec![
                        Span::styled(key_part.to_string(), Style::default().fg(Color::Cyan)),
                        Span::styled(val_part.to_string(), Style::default().fg(Color::White)),
                    ])
                } else {
                    Line::from(Span::styled(
                        src_line.to_string(),
                        Style::default().fg(Color::White),
                    ))
                };
                lines.push(line);
            }
            (Text::from(lines), true)
        } else {
            // Show TODO lines if any
            let mut lines: Vec<Line> = vec![Line::from(vec![
                Span::styled(
                    &entry.name,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  ({})", entry.status),
                    Style::default().fg(Color::DarkGray),
                ),
            ])];

            if entry.todo_lines.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  No TODOs",
                    Style::default().fg(Color::Green),
                )));
            } else {
                for todo in &entry.todo_lines {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", todo),
                        Style::default().fg(Color::Red),
                    )));
                }
            }

            (Text::from(lines), false)
        }
    } else {
        (
            Text::from(Line::from(Span::styled(
                "\u{2014}",
                Style::default().fg(Color::DarkGray),
            ))),
            false,
        )
    };

    let title = if has_dsl {
        " Detail \u{2014} J/K scroll "
    } else {
        " Detail "
    };

    let mut paragraph =
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(title));

    if has_dsl {
        paragraph = paragraph.scroll((app.cards_detail_scroll, 0));
    }

    f.render_widget(paragraph, area);
}
