//! Stack view — shows items on the stack when non-empty.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::PlayApp;

pub fn render(f: &mut Frame, app: &PlayApp, area: Rect) {
    if app.state.stack_objects.is_empty() {
        let empty = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<Line> = app
        .state
        .stack_objects
        .iter()
        .rev()
        .enumerate()
        .map(|(i, so)| {
            let desc = format!("[{}] {:?} (P{})", i + 1, so.kind, so.controller.0);
            Line::from(desc)
        })
        .collect();

    let stack = Paragraph::new(items)
        .block(
            Block::default()
                .title(" Stack ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(stack, area);
}
