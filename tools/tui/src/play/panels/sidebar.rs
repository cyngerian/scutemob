//! Player sidebar — shows all players with life totals and permanent counts.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::PlayApp;

pub fn render(f: &mut Frame, app: &PlayApp, area: Rect) {
    let players = app.state.active_players();

    let items: Vec<Line> = players
        .iter()
        .map(|&pid| {
            let life = app.state.player(pid).map(|p| p.life_total).unwrap_or(0);

            let perm_count = app.battlefield_objects(pid).len();

            let is_focused = pid == app.focused_player;
            let is_human = pid == app.human_player;

            let prefix = if is_focused { ">" } else { " " };
            let label = if is_human {
                format!("{}You  {:>3}  {}p", prefix, life, perm_count)
            } else {
                format!("{}P{:<2}  {:>3}  {}p", prefix, pid.0, life, perm_count)
            };

            let style = if is_focused {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if life <= 0 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::White)
            };

            Line::from(Span::styled(label, style))
        })
        .collect();

    let block = Block::default()
        .title(" Players ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let paragraph = Paragraph::new(items).block(block);
    f.render_widget(paragraph, area);
}
