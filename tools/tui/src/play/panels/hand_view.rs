//! Hand view — shows cards in the focused player's hand.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::PlayApp;

pub fn render(f: &mut Frame, app: &PlayApp, area: Rect) {
    let hand = app.hand_objects();
    let is_human = app.focused_player == app.human_player;

    let title = if is_human {
        " Your Hand ".to_string()
    } else {
        format!(" P{}'s Hand ({} cards) ", app.focused_player.0, hand.len())
    };

    let items: Vec<Line> = if !is_human {
        // Don't show other players' hands
        vec![Line::from(Span::styled(
            format!(" {} card(s) face down", hand.len()),
            Style::default().fg(Color::DarkGray),
        ))]
    } else if hand.is_empty() {
        vec![Line::from(Span::styled(
            " (empty hand)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        hand.iter()
            .enumerate()
            .map(|(i, (_id, name))| {
                let selected = i == app.selected_hand_idx;
                let prefix = if selected { ">" } else { " " };
                let style = if selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(
                    format!("{} [{}] {}", prefix, i + 1, name),
                    style,
                ))
            })
            .collect()
    };

    // Show mana pool and life under the hand
    let mut lines = items;
    if is_human {
        if let Ok(p) = app.state.player(app.human_player) {
            let pool = &p.mana_pool;
            let mana_text = format!(
                " Mana: W:{} U:{} B:{} R:{} G:{} C:{} | Life: {} | Lands: {}",
                pool.white,
                pool.blue,
                pool.black,
                pool.red,
                pool.green,
                pool.colorless,
                p.life_total,
                p.land_plays_remaining
            );
            lines.push(Line::from(Span::styled(
                mana_text,
                Style::default().fg(Color::Yellow),
            )));
        }
    }

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
