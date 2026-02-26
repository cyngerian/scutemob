use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::{theme, widgets::progress_bar::progress_bar};
use super::super::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &mut App) {
    // Split: summary (top, fixed) + scrollable list (bottom)
    let summary_height = (app.data.abilities.summary.len() as u16 * 2 + 2).min(12);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(summary_height), Constraint::Min(0)])
        .split(area);

    render_summary(f, chunks[0], app);
    render_ability_list(f, chunks[1], app);
}

fn render_summary(f: &mut Frame, area: Rect, app: &App) {
    let inner_width = area.width.saturating_sub(4);
    let mut lines: Vec<Line> = vec![];

    for row in &app.data.abilities.summary {
        if row.priority.to_lowercase().contains("total") || row.priority.is_empty() { continue; }
        let ratio = if row.total > 0 { row.validated as f64 / row.total as f64 } else { 0.0 };
        let label = format!("{}: {}/{} validated  complete:{} none:{}", row.priority, row.validated, row.total, row.complete, row.none);
        lines.push(progress_bar(ratio, inner_width, &label, theme::GREEN));
    }

    if lines.is_empty() {
        lines.push(Line::from("No data"));
    }

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Summary ")),
        area,
    );
}

fn render_ability_list(f: &mut Frame, area: Rect, app: &mut App) {
    let mut items: Vec<ListItem> = vec![];

    for section in &app.data.abilities.sections {
        // Section header
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("§ {}", section.name),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ])));

        // Two-column layout: pair rows side by side
        let rows = &section.rows;
        let mut i = 0;
        while i < rows.len() {
            let left = &rows[i];
            let right = rows.get(i + 1);

            let left_span = ability_row_spans(left);
            let right_span = right.map(ability_row_spans);

            let mut spans = left_span;
            if let Some(mut rs) = right_span {
                spans.push(Span::raw("  │  "));
                spans.append(&mut rs);
            }

            items.push(ListItem::new(Line::from(spans)));
            i += 2;
        }

        // Blank separator between sections
        items.push(ListItem::new(Line::from("")));
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from("No ability data")));
    }

    // Ensure selection is initialized
    if app.ability_list_state.selected().is_none() && !items.is_empty() {
        app.ability_list_state.select(Some(0));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Abilities (j/k to scroll) "))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(list, area, &mut app.ability_list_state);
}

fn ability_row_spans(row: &crate::dashboard::data::AbilityRow) -> Vec<Span<'static>> {
    let status_color = theme::status_color(&row.status);
    let symbol = theme::status_symbol(&row.status);
    vec![
        Span::styled(format!("{:<18}", row.name), Style::default().fg(Color::White)),
        Span::styled(format!("{:>2} ", row.priority), Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", symbol), Style::default().fg(status_color)),
        Span::styled(format!("{:<12}", row.status), Style::default().fg(status_color)),
    ]
}
