//! Event log — scrollable log of game events.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::PlayApp;

/// Pick a color for a log entry based on its text content.
fn event_color(text: &str) -> Color {
    if text.starts_with("Turn ") {
        Color::Cyan
    } else if text.contains("casts ") || text.contains("resolves") {
        Color::Rgb(180, 200, 255) // light blue for spell activity
    } else if text.contains("attacks") || text.contains("Combat damage") {
        Color::Rgb(255, 140, 100) // warm red for combat
    } else if text.contains("dies") || text.contains("loses") {
        Color::Rgb(255, 80, 80) // red for deaths
    } else if text.contains("discards") {
        Color::Rgb(200, 180, 140) // muted tan for discards
    } else if text.contains("enters the battlefield") || text.contains("plays ") {
        Color::Rgb(140, 230, 140) // green for permanents entering
    } else if text.contains("gains") {
        Color::Rgb(200, 255, 200) // light green for life gain
    } else if text.contains("Game Over") || text.contains("wins") {
        Color::Yellow
    } else {
        Color::DarkGray
    }
}

pub fn render(f: &mut Frame, app: &PlayApp, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize; // border

    // Auto-scroll to bottom, but allow manual scroll offset via j/k
    let auto_start = if app.event_log.len() > visible_height {
        app.event_log.len() - visible_height
    } else {
        0
    };

    // log_scroll=0 means "at the bottom" (auto-scroll), higher values scroll up
    let start = auto_start.saturating_sub(app.log_scroll);

    let items: Vec<Line> = app
        .event_log
        .iter()
        .skip(start)
        .take(visible_height)
        .map(|entry| {
            let color = event_color(&entry.text);
            Line::from(Span::styled(
                format!(" > {}", entry.text),
                Style::default().fg(color),
            ))
        })
        .collect();

    let block = Block::default()
        .title(" Event Log [j/k scroll] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(items).block(block);
    f.render_widget(paragraph, area);
}
