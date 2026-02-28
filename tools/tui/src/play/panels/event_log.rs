//! Event log — scrollable log of game events.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::PlayApp;

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
            Line::from(Span::styled(
                format!(" > {}", entry.text),
                Style::default().fg(Color::Gray),
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
