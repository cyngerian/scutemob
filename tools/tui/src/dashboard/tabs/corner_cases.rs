use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use super::super::app::App;
use crate::theme;

pub fn render(f: &mut Frame, area: Rect, app: &mut App) {
    let summary_height = 3u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(summary_height), Constraint::Min(0)])
        .split(area);

    render_summary(f, chunks[0], app);
    render_table(f, chunks[1], app);
}

fn render_summary(f: &mut Frame, area: Rect, app: &App) {
    let cc = &app.data.corner_cases;
    let total = cc.total();
    let filter_note = if app.show_gaps_only {
        "  [g: showing gaps only]"
    } else {
        "  [g: show all]"
    };

    let text = format!(
        "  Covered: {} ({:.0}%)  Gap: {} ({:.0}%)  Deferred: {} ({:.0}%)  Partial: {}{}",
        cc.covered,
        if total > 0 {
            cc.covered as f64 / total as f64 * 100.0
        } else {
            0.0
        },
        cc.gap,
        if total > 0 {
            cc.gap as f64 / total as f64 * 100.0
        } else {
            0.0
        },
        cc.deferred,
        if total > 0 {
            cc.deferred as f64 / total as f64 * 100.0
        } else {
            0.0
        },
        cc.partial,
        filter_note,
    );

    f.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::White)),
        area,
    );
}

fn render_table(f: &mut Frame, area: Rect, app: &mut App) {
    let cases: Vec<&crate::dashboard::data::CornerCase> = if app.show_gaps_only {
        app.data
            .corner_cases
            .cases
            .iter()
            .filter(|c| c.status == "GAP")
            .collect()
    } else {
        app.data.corner_cases.cases.iter().collect()
    };

    let header = Row::new(vec![
        Cell::from("#").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("CR Refs").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Milestone").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().fg(Color::White).bg(Color::DarkGray));

    let rows: Vec<Row> = cases
        .iter()
        .map(|c| {
            let status_color = theme::status_color(&c.status);
            let symbol = theme::status_symbol(&c.status);
            let status_text = format!("{}{}", symbol, c.status);

            let milestone_style = if c.milestone.is_empty() || c.milestone == "—" {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Gray)
            };
            let milestone_text = if c.milestone.is_empty() {
                "—".to_string()
            } else {
                c.milestone.clone()
            };

            Row::new(vec![
                Cell::from(c.number.to_string()),
                Cell::from(c.name.clone()),
                Cell::from(Span::styled(
                    c.cr_refs.clone(),
                    Style::default().fg(Color::DarkGray),
                )),
                Cell::from(Span::styled(status_text, Style::default().fg(status_color))),
                Cell::from(Span::styled(milestone_text, milestone_style)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Min(35),
        Constraint::Length(14),
        Constraint::Length(12),
        Constraint::Length(10),
    ];

    let title = if app.show_gaps_only {
        " Corner Cases — Gaps Only "
    } else {
        " Corner Cases (j/k:scroll  g:toggle gaps) "
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

    f.render_stateful_widget(table, area, &mut app.corner_case_table_state);
}
