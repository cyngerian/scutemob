//! Player sidebar — shows all players with life totals, permanent counts, and zone sizes.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::PlayApp;

pub fn render(f: &mut Frame, app: &PlayApp, area: Rect) {
    let players = app.state.active_players();

    let mut items: Vec<Line> = Vec::new();
    for &pid in &players {
        let life = app.state.player(pid).map(|p| p.life_total).unwrap_or(0);
        let perm_count = app.battlefield_objects(pid).len();
        let is_focused = pid == app.focused_player;
        let is_human = pid == app.human_player;

        let prefix = if is_focused { ">" } else { " " };
        let label = if is_human {
            format!("{}You  {:>3}hp  {}p", prefix, life, perm_count)
        } else {
            format!("{}P{:<2}  {:>3}hp  {}p", prefix, pid.0, life, perm_count)
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

        items.push(Line::from(Span::styled(label, style)));

        // Zone counts line: H:hand L:library G:graveyard E:exile
        let h = app.hand_count(pid);
        let l = app.library_count(pid);
        let g = app.graveyard_count(pid);
        let e = app.exile_count(pid);
        let zone_line = format!("  H:{} L:{} G:{} E:{}", h, l, g, e);
        items.push(Line::from(Span::styled(
            zone_line,
            Style::default().fg(Color::DarkGray),
        )));
    }

    let block = Block::default()
        .title(" Players ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let paragraph = Paragraph::new(items).block(block);
    f.render_widget(paragraph, area);
}
