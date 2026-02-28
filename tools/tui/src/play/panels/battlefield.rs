//! Battlefield panel — shows permanents for the focused player.
//! Lands in a compact horizontal row, non-lands vertically with P/T.

use ratatui::prelude::*;
use ratatui::widgets::*;

use mtg_engine::ObjectId;

use crate::play::app::{FocusZone, PlayApp};
use crate::play::panels::card_detail::card_color;

/// Look up an object's color from the game state.
fn color_for(app: &PlayApp, id: ObjectId) -> Color {
    app.state
        .object(id)
        .ok()
        .map(|obj| card_color(&obj.characteristics))
        .unwrap_or(Color::Gray)
}

pub fn render(f: &mut Frame, app: &PlayApp, area: Rect) {
    let player = app.focused_player;
    let is_human = player == app.human_player;
    let focused = app.focus_zone == FocusZone::Battlefield && is_human;

    let player_label = if is_human {
        "Your Battlefield".to_string()
    } else {
        format!("P{}'s Battlefield", player.0)
    };

    let lands = app.battlefield_lands(player);
    let nonlands = app.battlefield_nonlands(player);

    let mut lines: Vec<Line> = Vec::new();

    // Compact land row with colors
    if !lands.is_empty() {
        let mut spans: Vec<Span> = vec![Span::styled(
            " Lands: ",
            Style::default().fg(Color::DarkGray),
        )];
        for (i, (id, name, tapped)) in lands.iter().enumerate() {
            let color = color_for(app, *id);
            let label = if *tapped {
                format!("{}(T)", name)
            } else {
                name.clone()
            };
            let style = if *tapped {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(color)
            };
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(format!("[{}]", label), style));
        }
        lines.push(Line::from(spans));
    }

    // Non-lands vertically with P/T and colors
    if nonlands.is_empty() && lands.is_empty() {
        lines.push(Line::from(Span::styled(
            " (empty)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, (id, name, tapped, power, toughness)) in nonlands.iter().enumerate() {
            let selected = i == app.selected_bf_idx && is_human;
            let prefix = if selected { ">" } else { " " };
            let pt = match (power, toughness) {
                (Some(p), Some(t)) => format!(" {}/{}", p, t),
                _ => String::new(),
            };
            let tap_indicator = if *tapped { " (T)" } else { "" };

            let color = color_for(app, *id);

            let style = if selected {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else if *tapped {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(color)
            };
            lines.push(Line::from(Span::styled(
                format!("{} [{}{}]{}", prefix, name, pt, tap_indicator),
                style,
            )));
        }
    }

    let border_color = if focused { Color::Cyan } else { Color::Green };
    let block = Block::default()
        .title(format!(" {} ", player_label))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
