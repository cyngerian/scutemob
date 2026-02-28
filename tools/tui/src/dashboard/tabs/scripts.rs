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
            Constraint::Length(4), // detail (2 content lines + 2 borders)
        ])
        .split(area);

    render_summary(f, chunks[0], app);
    render_table(f, chunks[1], app);
    render_detail(f, chunks[2], app);
}

fn render_summary(f: &mut Frame, area: Rect, app: &App) {
    let s = &app.data.scripts;
    let filter_label = if app.scripts_show_pending_only {
        " [p: pending only ← a: show all]"
    } else {
        " [p: pending only]"
    };

    let line = Line::from(vec![
        Span::styled("Total: ", Style::default().fg(Color::Gray)),
        Span::styled(
            s.total.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled("Approved: ", Style::default().fg(Color::Gray)),
        Span::styled(s.approved.to_string(), Style::default().fg(theme::GREEN)),
        Span::raw("   "),
        Span::styled("Pending review: ", Style::default().fg(Color::Gray)),
        Span::styled(
            s.pending_review.to_string(),
            if s.pending_review > 0 {
                Style::default().fg(theme::GOLD)
            } else {
                Style::default().fg(theme::GREEN)
            },
        ),
        Span::styled(filter_label, Style::default().fg(Color::DarkGray)),
    ]);

    f.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::ALL).title(" Summary ")),
        area,
    );
}

fn render_table(f: &mut Frame, area: Rect, app: &mut App) {
    let entries: Vec<&crate::dashboard::data::ScriptEntry> = app
        .data
        .scripts
        .entries
        .iter()
        .filter(|e| !app.scripts_show_pending_only || e.status == "pending_review")
        .collect();

    let header = Row::new(vec![
        Cell::from("").style(Style::default().add_modifier(Modifier::BOLD)), // status symbol
        Cell::from("Dir").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Script").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Chks").style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Gray),
        ),
    ])
    .style(Style::default().fg(Color::White).bg(Color::DarkGray));

    let rows: Vec<Row> = entries
        .iter()
        .map(|e| {
            let (symbol, sym_style) = match e.status.as_str() {
                "approved" => ("✓", Style::default().fg(theme::GREEN)),
                "pending_review" => ("●", Style::default().fg(theme::GOLD)),
                _ => ("?", Style::default().fg(Color::DarkGray)),
            };

            let dir_style = Style::default().fg(Color::Gray);
            let name_style = match e.status.as_str() {
                "pending_review" => Style::default().fg(theme::GOLD),
                _ => Style::default().fg(Color::White),
            };
            let chk_style = Style::default().fg(Color::DarkGray);

            Row::new(vec![
                Cell::from(Span::styled(symbol, sym_style)),
                Cell::from(Span::styled(e.directory.clone(), dir_style)),
                Cell::from(Span::styled(e.filename.clone(), name_style)),
                Cell::from(Span::styled(format!("{:>3}", e.assertion_count), chk_style)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(2),  // symbol
        Constraint::Length(13), // dir (max: "etb-triggers" = 12)
        Constraint::Min(20),    // filename
        Constraint::Length(4),  // assertion count
    ];

    let visible = entries.len();
    let title = if app.scripts_show_pending_only {
        format!(" Scripts — pending review ({}) ", visible)
    } else {
        format!(" Scripts ({}) — j/k scroll ", visible)
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("→ ");

    f.render_stateful_widget(table, area, &mut app.scripts_table_state);
}

fn render_detail(f: &mut Frame, area: Rect, app: &App) {
    let entries: Vec<&crate::dashboard::data::ScriptEntry> = app
        .data
        .scripts
        .entries
        .iter()
        .filter(|e| !app.scripts_show_pending_only || e.status == "pending_review")
        .collect();

    let sel = app.scripts_table_state.selected().unwrap_or(0);
    let text = if let Some(entry) = entries.get(sel) {
        let (sym, sym_style) = match entry.status.as_str() {
            "approved" => ("✓", Style::default().fg(theme::GREEN)),
            "pending_review" => ("●", Style::default().fg(theme::GOLD)),
            _ => ("?", Style::default().fg(Color::DarkGray)),
        };
        let chk_label = if entry.assertion_count == 1 {
            "1 check".to_string()
        } else {
            format!("{} checks", entry.assertion_count)
        };
        // Line 1: symbol + path + assertion count
        let line1 = Line::from(vec![
            Span::styled(format!("{} ", sym), sym_style),
            Span::styled(
                format!("{}/{}  ", entry.directory, entry.filename),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(chk_label, Style::default().fg(Color::DarkGray)),
        ]);
        // Line 2: indented full scenario name
        let line2 = Line::from(vec![
            Span::raw("  "),
            Span::styled(&entry.name, Style::default().fg(Color::White)),
        ]);
        Text::from(vec![line1, line2])
    } else {
        Text::from(Line::from(Span::styled(
            "—",
            Style::default().fg(Color::DarkGray),
        )))
    };

    f.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Description "),
        ),
        area,
    );
}
