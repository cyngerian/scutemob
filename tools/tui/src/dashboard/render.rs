use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::app::{App, LiveTestCount, TAB_NAMES};
use super::tabs;

/// Minimum inner width (excluding borders) needed to fit all tabs on one line.
fn tabs_single_row_min_width() -> u16 {
    let total: usize = TAB_NAMES
        .iter()
        .enumerate()
        .map(|(i, n)| n.len() + if i + 1 < TAB_NAMES.len() { 3 } else { 0 })
        .sum();
    total as u16
}

/// Build the tab bar lines, wrapping when `inner_width` is insufficient.
fn build_tab_lines(current_tab: usize, inner_width: u16) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = vec![];
    let mut spans: Vec<Span<'static>> = vec![];
    let mut row_len: usize = 0;
    let avail = inner_width as usize;

    for (i, &name) in TAB_NAMES.iter().enumerate() {
        let sep = if i + 1 < TAB_NAMES.len() { " │ " } else { "" };
        let entry_len = name.len() + sep.len();

        if !spans.is_empty() && row_len + entry_len > avail {
            lines.push(Line::from(spans.clone()));
            spans.clear();
            row_len = 0;
        }

        let style = if i == current_tab {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(name.to_string(), style));
        row_len += name.len();

        if !sep.is_empty() {
            spans.push(Span::styled(
                sep.to_string(),
                Style::default().fg(Color::DarkGray),
            ));
            row_len += sep.len();
        }
    }
    if !spans.is_empty() {
        lines.push(Line::from(spans));
    }
    lines
}

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let inner_width = area.width.saturating_sub(2);
    let needs_wrap = inner_width < tabs_single_row_min_width();
    let tabs_height: u16 = if needs_wrap { 4 } else { 3 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(tabs_height),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // ─── tabs bar ───────────────────────────────────────────────────────────
    let tab_lines = build_tab_lines(app.current_tab, inner_width);
    let tabs_widget = Paragraph::new(Text::from(tab_lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" MTG Commander Rules Engine "),
    );
    f.render_widget(tabs_widget, chunks[0]);

    // ─── tab content ────────────────────────────────────────────────────────
    match app.current_tab {
        0 => tabs::dashboard::render(f, chunks[1], app),
        1 => tabs::pipeline::render(f, chunks[1], app),
        2 => tabs::cards::render(f, chunks[1], app),
        3 => tabs::milestones::render(f, chunks[1], app),
        _ => {}
    }

    // ─── status bar ─────────────────────────────────────────────────────────
    let help = match app.current_tab {
        0 => "q:quit  Tab:next  1-4:jump  r:refresh",
        1 => "q:quit  Tab:next  j/k:scroll  r:refresh",
        2 => "q:quit  Tab:next  j/k:scroll  J/K:detail  t:todo  o:ok  p:partial  s:stripped  a:all",
        3 => "q:quit  Tab:next  j/k:scroll  r:refresh",
        _ => "q:quit",
    };
    let test_str = match &app.live_test_count {
        LiveTestCount::Loading => "...".to_string(),
        LiveTestCount::Done(n) => n.to_string(),
    };
    let status_text = format!(
        " {:<80} Tests: {}  Scripts: {} ",
        help, test_str, app.data.scripts.total,
    );
    f.render_widget(
        ratatui::widgets::Paragraph::new(status_text)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray)),
        chunks[2],
    );
}
