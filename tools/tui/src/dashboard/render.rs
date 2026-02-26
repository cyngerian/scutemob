use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Tabs},
    Frame,
};

use super::app::{App, TAB_NAMES};
use super::tabs;

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // ─── outer layout: tabs bar (3 lines) + content + status bar (1 line) ──
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs bar
            Constraint::Min(0),    // content
            Constraint::Length(1), // status bar
        ])
        .split(area);

    // ─── tabs bar ───────────────────────────────────────────────────────────
    let tab_titles: Vec<Span> = TAB_NAMES
        .iter()
        .map(|t| Span::raw(*t))
        .collect();

    let tabs_widget = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" MTG Commander Rules Engine — Progress Dashboard "),
        )
        .select(app.current_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs_widget, chunks[0]);

    // ─── tab content ────────────────────────────────────────────────────────
    match app.current_tab {
        0 => tabs::overview::render(f, chunks[1], app),
        1 => tabs::milestones::render(f, chunks[1], app),
        2 => tabs::abilities::render(f, chunks[1], app),
        3 => tabs::corner_cases::render(f, chunks[1], app),
        4 => tabs::reviews::render(f, chunks[1], app),
        _ => {}
    }

    // ─── status bar ─────────────────────────────────────────────────────────
    let help = match app.current_tab {
        0 => "q:quit  Tab:next  1-5:jump  r:refresh",
        1 => "q:quit  Tab:next  j/k:scroll  r:refresh",
        2 => "q:quit  Tab:next  j/k:scroll  r:refresh",
        3 => "q:quit  Tab:next  j/k:scroll  g:gaps only  r:refresh",
        4 => "q:quit  Tab:next  r:refresh",
        _ => "q:quit",
    };
    let status_text = format!(
        " {:<60} Active: {}  Tests: {}  Scripts: {} ",
        help,
        app.data.current_state.active_milestone,
        app.data.current_state.test_count,
        app.data.current_state.script_count,
    );
    f.render_widget(
        ratatui::widgets::Paragraph::new(status_text)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray)),
        chunks[2],
    );
}
